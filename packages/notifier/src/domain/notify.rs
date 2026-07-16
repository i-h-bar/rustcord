const BATCH_SIZE: i64 = 10;

/// After this many consecutive *webhook/subscription-specific* failures
/// (see `SpoilerError::counts_toward_failure_threshold`) — invalid or
/// revoked webhook token, missing permissions — assume the webhook is
/// permanently dead and unsubscribe it automatically rather than retrying
/// forever. A clean 404 skips this threshold entirely (see
/// `SpoilerError::is_permanently_gone`), and errors that could just as
/// easily mean Discord itself is having trouble (connection errors, 5xx,
/// 429) never count towards this at all, so an outage can never push a
/// healthy subscription toward deletion. The streak resets to `0` on every
/// successful delivery (folded into `ack`).
const MAX_CONSECUTIVE_FAILURES: i64 = 5;

pub async fn run(
    repo: &impl cards_sdk::SpoilerQueue,
    images: &impl crate::ports::images::ImageStore,
    sender: &impl crate::ports::spoilers::SpoilerSender,
) {
    for sub in repo.subscriptions_with_pending().await {
        let cards = repo
            .pending_cards(sub.guild_id, sub.channel_id, BATCH_SIZE)
            .await;

        // The cursor is a single scalar, so a card whose image can't be
        // fetched yet can't be skipped over: acking past it would drop it
        // forever, and sending later cards without acking it would resend
        // them next poll (since the query is `id > cursor`). So the batch
        // stops at the first missing image rather than skipping it.
        let mut batch = Vec::with_capacity(cards.len());
        let mut last_queue_id = None;
        for pending in cards {
            match images.fetch(&pending.card).await {
                Ok(bytes) => {
                    last_queue_id = Some(pending.queue_id);
                    batch.push((pending.card, bytes));
                }
                Err(e) => {
                    log::warn!(
                        "Stopping batch for guild {} at missing image for {}: {e}",
                        sub.guild_id,
                        pending.card.name()
                    );
                    break;
                }
            }
        }

        let Some(last_queue_id) = last_queue_id else {
            continue;
        };

        match sender.send(sub.id, &sub.token, &batch).await {
            Ok(()) => repo.ack(sub.guild_id, sub.channel_id, last_queue_id).await,
            Err(e) => {
                log::warn!(
                    "Failed to deliver batch to guild {} channel {}: {e}",
                    sub.guild_id,
                    sub.channel_id
                );
                if e.is_permanently_gone() {
                    log::warn!(
                        "Webhook for guild {} channel {} is gone (404) — unsubscribing immediately",
                        sub.guild_id,
                        sub.channel_id
                    );
                    repo.delete_subscription(sub.guild_id, sub.channel_id).await;
                } else if e.counts_toward_failure_threshold() {
                    let failures = repo.record_failure(sub.guild_id, sub.channel_id).await;
                    if failures >= MAX_CONSECUTIVE_FAILURES {
                        log::warn!(
                            "Auto-unsubscribing guild {} channel {} after {failures} consecutive webhook failures",
                            sub.guild_id,
                            sub.channel_id
                        );
                        repo.delete_subscription(sub.guild_id, sub.channel_id).await;
                    }
                }
                // else: a connection error, 5xx, or rate limiting — could
                // just as easily mean Discord itself is having trouble, so
                // it's neither recorded nor counted toward unsubscribing.
                // The batch is simply retried next poll.
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
    use crate::ports::images::MockImageStore;
    use crate::ports::spoilers::{MockSpoilerSender, SpoilerError};
    use cards_sdk::spoiler::{PendingCard, Subscription};
    use cards_sdk::{ChannelId, GuildId, MockSpoilerQueue, SubscriptionId};
    use mockall::predicate::eq;

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

    fn always_returns_image_bytes() -> MockImageStore {
        let mut images = MockImageStore::new();
        images.expect_fetch().returning(|_| Ok(vec![1, 2, 3]));
        images
    }

    fn test_subscription() -> Subscription {
        Subscription {
            guild_id: GuildId::from(1u64),
            channel_id: ChannelId::from(2u64),
            id: SubscriptionId::from(3u64),
            token: "tok".to_string(),
            cursor: 0,
        }
    }

    #[tokio::test]
    async fn acks_up_to_the_last_card_after_a_successful_batch_send() {
        let mut repo = MockSpoilerQueue::new();
        let mut sender = MockSpoilerSender::new();
        let images = always_returns_image_bytes();

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| vec![test_subscription()]);
        repo.expect_pending_cards()
            .with(
                eq(GuildId::from(1u64)),
                eq(ChannelId::from(2u64)),
                eq(10i64),
            )
            .times(1)
            .return_once(|_, _, _| {
                vec![
                    PendingCard {
                        queue_id: 1,
                        card: test_card("Card A"),
                    },
                    PendingCard {
                        queue_id: 2,
                        card: test_card("Card B"),
                    },
                ]
            });
        sender
            .expect_send()
            .withf(|_, _, cards| cards.len() == 2)
            .times(1)
            .returning(|_, _, _| Ok(()));
        repo.expect_ack()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(2u64)), eq(2i64))
            .times(1)
            .return_const(());
        repo.expect_prune_queue().times(1).return_const(());

        run(&repo, &images, &sender).await;
    }

    #[tokio::test]
    async fn does_not_ack_and_records_a_failure_when_the_batch_send_fails() {
        let mut repo = MockSpoilerQueue::new();
        let mut sender = MockSpoilerSender::new();
        let images = always_returns_image_bytes();

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| vec![test_subscription()]);
        repo.expect_pending_cards()
            .with(
                eq(GuildId::from(1u64)),
                eq(ChannelId::from(2u64)),
                eq(10i64),
            )
            .times(1)
            .return_once(|_, _, _| {
                vec![
                    PendingCard {
                        queue_id: 1,
                        card: test_card("Card A"),
                    },
                    PendingCard {
                        queue_id: 2,
                        card: test_card("Card B"),
                    },
                ]
            });
        sender
            .expect_send()
            .times(1)
            .returning(|_, _, _| Err(SpoilerError::Status(403)));
        repo.expect_ack().times(0);
        // Below MAX_CONSECUTIVE_FAILURES (5) — must NOT trigger auto-unsubscribe.
        repo.expect_record_failure()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(2u64)))
            .times(1)
            .return_once(|_, _| 2);
        repo.expect_delete_subscription().times(0);
        repo.expect_prune_queue().times(1).return_const(());

        run(&repo, &images, &sender).await;
    }

    #[tokio::test]
    async fn auto_unsubscribes_once_the_failure_threshold_is_reached() {
        let mut repo = MockSpoilerQueue::new();
        let mut sender = MockSpoilerSender::new();
        let images = always_returns_image_bytes();

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| vec![test_subscription()]);
        repo.expect_pending_cards()
            .with(
                eq(GuildId::from(1u64)),
                eq(ChannelId::from(2u64)),
                eq(10i64),
            )
            .times(1)
            .return_once(|_, _, _| {
                vec![PendingCard {
                    queue_id: 1,
                    card: test_card("Card A"),
                }]
            });
        sender
            .expect_send()
            .times(1)
            .returning(|_, _, _| Err(SpoilerError::Status(403)));
        repo.expect_ack().times(0);
        // Reaches MAX_CONSECUTIVE_FAILURES (5) exactly — must trigger auto-unsubscribe.
        repo.expect_record_failure()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(2u64)))
            .times(1)
            .return_once(|_, _| 5);
        repo.expect_delete_subscription()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(2u64)))
            .times(1)
            .return_const(Some(SubscriptionId::from(3u64)));
        repo.expect_prune_queue().times(1).return_const(());

        run(&repo, &images, &sender).await;
    }

    #[tokio::test]
    async fn immediately_unsubscribes_on_a_404_without_waiting_for_the_threshold() {
        let mut repo = MockSpoilerQueue::new();
        let mut sender = MockSpoilerSender::new();
        let images = always_returns_image_bytes();

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| vec![test_subscription()]);
        repo.expect_pending_cards()
            .with(
                eq(GuildId::from(1u64)),
                eq(ChannelId::from(2u64)),
                eq(10i64),
            )
            .times(1)
            .return_once(|_, _, _| {
                vec![PendingCard {
                    queue_id: 1,
                    card: test_card("Card A"),
                }]
            });
        sender
            .expect_send()
            .times(1)
            .returning(|_, _, _| Err(SpoilerError::Status(404)));
        repo.expect_ack().times(0);
        // A 404 skips the failure-counting mechanism entirely.
        repo.expect_record_failure().times(0);
        repo.expect_delete_subscription()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(2u64)))
            .times(1)
            .return_const(Some(SubscriptionId::from(3u64)));
        repo.expect_prune_queue().times(1).return_const(());

        run(&repo, &images, &sender).await;
    }

    #[tokio::test]
    async fn a_transient_error_never_counts_toward_the_failure_threshold() {
        let mut repo = MockSpoilerQueue::new();
        let mut sender = MockSpoilerSender::new();
        let images = always_returns_image_bytes();

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| vec![test_subscription()]);
        repo.expect_pending_cards()
            .with(
                eq(GuildId::from(1u64)),
                eq(ChannelId::from(2u64)),
                eq(10i64),
            )
            .times(1)
            .return_once(|_, _, _| {
                vec![PendingCard {
                    queue_id: 1,
                    card: test_card("Card A"),
                }]
            });
        // A 503 (or a connection error) could just as easily mean Discord
        // itself is down — must never be recorded as this guild's failure.
        sender
            .expect_send()
            .times(1)
            .returning(|_, _, _| Err(SpoilerError::Status(503)));
        repo.expect_ack().times(0);
        repo.expect_record_failure().times(0);
        repo.expect_delete_subscription().times(0);
        repo.expect_prune_queue().times(1).return_const(());

        run(&repo, &images, &sender).await;
    }

    #[tokio::test]
    async fn stops_the_batch_at_the_first_missing_image_and_acks_only_the_cards_before_it() {
        let mut repo = MockSpoilerQueue::new();
        let mut sender = MockSpoilerSender::new();
        let mut images = MockImageStore::new();

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| vec![test_subscription()]);
        repo.expect_pending_cards()
            .with(
                eq(GuildId::from(1u64)),
                eq(ChannelId::from(2u64)),
                eq(10i64),
            )
            .times(1)
            .return_once(|_, _, _| {
                vec![
                    PendingCard {
                        queue_id: 1,
                        card: test_card("Card A"),
                    },
                    PendingCard {
                        queue_id: 2,
                        card: test_card("Missing Image"),
                    },
                    // Must never be reached — the batch stops at the gap
                    // left by card 2, so this card isn't even fetched.
                    PendingCard {
                        queue_id: 3,
                        card: test_card("Card C"),
                    },
                ]
            });
        images
            .expect_fetch()
            .times(1)
            .returning(|_| Ok(vec![1, 2, 3]));
        images.expect_fetch().times(1).return_once(|_| {
            Err(crate::ports::images::ImageRetrievalError::new(
                "no file".into(),
            ))
        });
        sender
            .expect_send()
            .withf(|_, _, cards| cards.len() == 1)
            .times(1)
            .returning(|_, _, _| Ok(()));
        repo.expect_ack()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(2u64)), eq(1i64))
            .times(1)
            .return_const(());
        repo.expect_prune_queue().times(1).return_const(());

        run(&repo, &images, &sender).await;
    }

    #[tokio::test]
    async fn sends_nothing_when_the_first_card_has_no_image() {
        let mut repo = MockSpoilerQueue::new();
        let sender = MockSpoilerSender::new();
        let mut images = MockImageStore::new();

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| vec![test_subscription()]);
        repo.expect_pending_cards()
            .with(
                eq(GuildId::from(1u64)),
                eq(ChannelId::from(2u64)),
                eq(10i64),
            )
            .times(1)
            .return_once(|_, _, _| {
                vec![PendingCard {
                    queue_id: 1,
                    card: test_card("Missing Image"),
                }]
            });
        images.expect_fetch().times(1).return_once(|_| {
            Err(crate::ports::images::ImageRetrievalError::new(
                "no file".into(),
            ))
        });
        // `sender` has no expectations at all — send() must never be called.
        repo.expect_ack().times(0);
        repo.expect_prune_queue().times(1).return_const(());

        run(&repo, &images, &sender).await;
    }
}
