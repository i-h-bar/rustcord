use discord_embeds::create_embed_with_image_url;

const BATCH_SIZE: i64 = 10;

/// After this many consecutive `notifier` runs fail to deliver to a guild,
/// assume its webhook is permanently dead (channel/webhook deleted,
/// permissions revoked) and unsubscribe it automatically rather than
/// retrying forever — resolves the spec's "auto-unsubscribe vs. retry
/// indefinitely" open question in favour of auto-unsubscribe. The streak
/// resets to `0` on every successful delivery (folded into `ack`).
const MAX_CONSECUTIVE_FAILURES: i64 = 5;

pub async fn run(
    repo: &impl cards_sdk::SpoilerQueue,
    sender: &impl crate::ports::webhook::WebhookSender,
) {
    for sub in repo.subscriptions_with_pending().await {
        let cards = repo.pending_cards(sub.guild_id, BATCH_SIZE).await;
        for pending in cards {
            let embed = create_embed_with_image_url(&pending.card, &pending.image_url).await;
            match sender.send(sub.webhook_id, &sub.webhook_token, embed).await {
                Ok(()) => repo.ack(sub.guild_id, pending.queue_id).await,
                Err(e) => {
                    log::warn!(
                        "Stopping delivery to guild {} after webhook failure: {e}",
                        sub.guild_id
                    );
                    let failures = repo.record_failure(sub.guild_id).await;
                    if failures >= MAX_CONSECUTIVE_FAILURES {
                        log::warn!(
                            "Auto-unsubscribing guild {} after {failures} consecutive webhook failures",
                            sub.guild_id
                        );
                        repo.delete_subscription(sub.guild_id).await;
                    }
                    break;
                }
            }
        }
    }

    // Runs once per invocation, after every subscription's cursor has been
    // advanced as far as this run could get it — see cards_sdk's
    // `SpoilerQueue::prune_queue` doc comment for the safety argument
    // (only rows below/at the *minimum* cursor across all subscriptions
    // ever get deleted, so a lagging guild is never skipped).
    repo.prune_queue().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::webhook::{MockWebhookSender, WebhookError};
    use cards_sdk::MockSpoilerQueue;
    use cards_sdk::spoiler::{PendingCard, Subscription};
    use mockall::predicate::eq;

    /// `discord_embeds::create_embed_with_image_url` lazily initializes a
    /// process-wide emoji cache that requires `BOT_TOKEN` to be present
    /// (network calls inside it degrade gracefully on failure, so a dummy
    /// value is fine for tests — only the `env::var(...).expect(...)` at
    /// startup is fatal if unset). `std::sync::Once` guarantees this runs
    /// exactly once and that every caller — including tests running on
    /// other threads — blocks until it's done, so it's safe under `cargo
    /// test`'s parallel test execution.
    fn ensure_bot_token_env() {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            // SAFETY: guarded by `Once` above; nothing else in this test
            // binary reads or writes `BOT_TOKEN` concurrently.
            unsafe { std::env::set_var("BOT_TOKEN", "test-token") };
        });
    }

    fn test_card(name: &str) -> contracts::card::Card {
        let id = uuid::Uuid::new_v4();
        contracts::card::Card::new(
            id,
            name.to_string(),
            name.to_lowercase(),
            id,
            "https://scryfall.com/card/x/1".to_string(),
            id,
            None,
            "{R}".to_string(),
            vec!["R".to_string()],
            None,
            None,
            None,
            None,
            "Instant".to_string(),
            "Deal damage".to_string(),
            None,
            "Artist".to_string(),
            "Alpha".to_string(),
            "LEA".to_string(),
            time::Date::from_calendar_date(1993, time::Month::August, 5).unwrap(),
        )
    }

    #[tokio::test]
    async fn acks_each_card_after_successful_delivery() {
        ensure_bot_token_env();
        let mut repo = MockSpoilerQueue::new();
        let mut sender = MockWebhookSender::new();

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| {
                vec![Subscription {
                    guild_id: 1,
                    channel_id: 2,
                    webhook_id: 3,
                    webhook_token: "tok".to_string(),
                    cursor: 0,
                }]
            });
        repo.expect_pending_cards()
            .with(eq(1i64), eq(10i64))
            .times(1)
            .return_once(|_, _| {
                vec![
                    PendingCard {
                        queue_id: 1,
                        card: test_card("Card A"),
                        image_url: "u1".to_string(),
                    },
                    PendingCard {
                        queue_id: 2,
                        card: test_card("Card B"),
                        image_url: "u2".to_string(),
                    },
                ]
            });
        sender.expect_send().times(2).returning(|_, _, _| Ok(()));
        repo.expect_ack()
            .with(eq(1i64), eq(1i64))
            .times(1)
            .return_const(());
        repo.expect_ack()
            .with(eq(1i64), eq(2i64))
            .times(1)
            .return_const(());
        repo.expect_prune_queue().times(1).return_const(());

        run(&repo, &sender).await;
    }

    #[tokio::test]
    async fn stops_processing_a_guild_after_the_first_failure_mid_batch() {
        ensure_bot_token_env();
        let mut repo = MockSpoilerQueue::new();
        let mut sender = MockWebhookSender::new();

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| {
                vec![Subscription {
                    guild_id: 1,
                    channel_id: 2,
                    webhook_id: 3,
                    webhook_token: "tok".to_string(),
                    cursor: 0,
                }]
            });
        repo.expect_pending_cards()
            .with(eq(1i64), eq(10i64))
            .times(1)
            .return_once(|_, _| {
                vec![
                    PendingCard {
                        queue_id: 1,
                        card: test_card("Card A"),
                        image_url: "u1".to_string(),
                    },
                    PendingCard {
                        queue_id: 2,
                        card: test_card("Card B"),
                        image_url: "u2".to_string(),
                    },
                    // A third card that must never be attempted — it's only
                    // reachable if the failure on card 2 doesn't stop the
                    // batch, which is exactly the behaviour under test.
                    PendingCard {
                        queue_id: 3,
                        card: test_card("Card C"),
                        image_url: "u3".to_string(),
                    },
                ]
            });
        // First send succeeds and is acked; second fails and must NOT be acked;
        // exactly 2 sends total (mockall panics on a 3rd call, since no
        // expectation covers it) — proves the batch stopped after card 2.
        sender.expect_send().times(1).returning(|_, _, _| Ok(()));
        sender
            .expect_send()
            .times(1)
            .returning(|_, _, _| Err(WebhookError::Status(404)));
        repo.expect_ack()
            .with(eq(1i64), eq(1i64))
            .times(1)
            .return_const(());
        repo.expect_ack()
            .with(eq(1i64), eq(2i64))
            .times(0)
            .return_const(());
        repo.expect_ack()
            .with(eq(1i64), eq(3i64))
            .times(0)
            .return_const(());
        // Below MAX_CONSECUTIVE_FAILURES (5) — must NOT trigger auto-unsubscribe.
        repo.expect_record_failure()
            .with(eq(1i64))
            .times(1)
            .return_once(|_| 2);
        repo.expect_delete_subscription().times(0);
        repo.expect_prune_queue().times(1).return_const(());

        run(&repo, &sender).await;
    }

    #[tokio::test]
    async fn auto_unsubscribes_once_the_failure_threshold_is_reached() {
        ensure_bot_token_env();
        let mut repo = MockSpoilerQueue::new();
        let mut sender = MockWebhookSender::new();

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| {
                vec![Subscription {
                    guild_id: 1,
                    channel_id: 2,
                    webhook_id: 3,
                    webhook_token: "tok".to_string(),
                    cursor: 0,
                }]
            });
        repo.expect_pending_cards()
            .with(eq(1i64), eq(10i64))
            .times(1)
            .return_once(|_, _| {
                vec![PendingCard {
                    queue_id: 1,
                    card: test_card("Card A"),
                    image_url: "u1".to_string(),
                }]
            });
        sender
            .expect_send()
            .times(1)
            .returning(|_, _, _| Err(WebhookError::Status(404)));
        repo.expect_ack().times(0);
        // Reaches MAX_CONSECUTIVE_FAILURES (5) exactly — must trigger auto-unsubscribe.
        repo.expect_record_failure()
            .with(eq(1i64))
            .times(1)
            .return_once(|_| 5);
        repo.expect_delete_subscription()
            .with(eq(1i64))
            .times(1)
            .return_const(Some(3i64));
        repo.expect_prune_queue().times(1).return_const(());

        run(&repo, &sender).await;
    }
}
