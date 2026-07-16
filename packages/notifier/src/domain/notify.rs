use cards_sdk::spoiler::{PendingCard, Subscription};
use futures::stream::{self, StreamExt};
use std::collections::HashMap;
use uuid::Uuid;

const BATCH_SIZE: usize = 10;

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

/// How many subscriptions are delivered to concurrently in one poll.
/// Different subscriptions target entirely different webhooks, so there's
/// no rate-limit interaction between them. Card/image fetching already
/// happens once, up front, outside this concurrency entirely — what's
/// actually concurrent here is each subscription's own `ack`/
/// `record_failure`/`delete_subscription` bookkeeping call, which *does*
/// hit `cards_sdk::Postgres`. This is capped near that pool's default
/// connection count (5) purely so a large poll doesn't cause needless
/// contention for a connection to run those. Sends *within* one
/// subscription always stay sequential regardless of this limit (see
/// `deliver_to_subscription`).
const MAX_CONCURRENT_DELIVERIES: usize = 5;

pub async fn run(
    repo: &impl cards_sdk::SpoilerQueue,
    images: &impl crate::ports::images::ImageStore,
    sender: &impl crate::ports::spoilers::SpoilerSender,
) {
    let subs = repo.subscriptions_with_pending().await;

    if let Some(min_cursor) = subs.iter().map(|sub| sub.cursor).min() {
        // Fetched once per poll and shared across every subscription below —
        // most subscriptions are behind on the same cards, so this avoids
        // every one of them re-running the same joined query individually.
        let shared_cards = repo.pending_cards_since(min_cursor).await;

        // Resolved once per unique image, up front, before any subscription
        // starts sending — turns the concurrent delivery phase below into a
        // read-only lookup, so it needs no locking around the cache.
        let image_cache = resolve_images(&subs, &shared_cards, images).await;

        stream::iter(subs.iter())
            .for_each_concurrent(MAX_CONCURRENT_DELIVERIES, |sub| {
                deliver_to_subscription(repo, sender, sub, &shared_cards, &image_cache)
            })
            .await;
    }

    // Runs once per invocation, after every subscription's cursor has been
    // advanced as far as this run could get it — see cards_sdk's
    // `SpoilerQueue::prune_queue` doc comment for the safety argument
    // (only rows below/at the *minimum* cursor across all subscriptions
    // ever get deleted, so a lagging guild is never skipped).
    repo.prune_queue().await;
}

/// Resolves every image that at least one subscription's own
/// cursor-filtered, `BATCH_SIZE`-capped window will actually need, each
/// fetched from disk at most once regardless of how many subscriptions
/// share it. A failed fetch is cached as `None` (and logged once here,
/// rather than once per subscription that would have hit it).
async fn resolve_images(
    subs: &[Subscription],
    shared_cards: &[PendingCard],
    images: &impl crate::ports::images::ImageStore,
) -> HashMap<Uuid, Option<Vec<u8>>> {
    let mut cache = HashMap::new();

    for sub in subs {
        let pending_for_sub = shared_cards
            .iter()
            .filter(|pending| pending.queue_id > sub.cursor)
            .take(BATCH_SIZE);

        for pending in pending_for_sub {
            let image_id = *pending.card.image_id();
            if cache.contains_key(&image_id) {
                continue;
            }

            let result = match images.fetch(&pending.card).await {
                Ok(bytes) => Some(bytes),
                Err(e) => {
                    log::warn!("Missing image for card {}: {e}", pending.card.name());
                    None
                }
            };
            cache.insert(image_id, result);
        }
    }

    cache
}

/// Delivers as many of `sub`'s pending cards as it can, one message at a
/// time, and acks up to the last one actually delivered. Stops at the first
/// missing image or send failure — a missing image is already known by this
/// point (see `resolve_images`) rather than being fetched here.
async fn deliver_to_subscription(
    repo: &impl cards_sdk::SpoilerQueue,
    sender: &impl crate::ports::spoilers::SpoilerSender,
    sub: &Subscription,
    shared_cards: &[PendingCard],
    image_cache: &HashMap<Uuid, Option<Vec<u8>>>,
) {
    // The cursor is a single scalar, so a card whose image is missing, or
    // whose send fails, can't be skipped over: acking past it would drop it
    // forever, and sending later cards without acking it would resend them
    // next poll. So this stops at the first problem rather than skipping it
    // — everything before that point has already been delivered as its own
    // message, so it's acked regardless of what happens next.
    let mut last_queue_id = None;
    let pending_for_sub = shared_cards
        .iter()
        .filter(|pending| pending.queue_id > sub.cursor)
        .take(BATCH_SIZE);

    for pending in pending_for_sub {
        let image_id = *pending.card.image_id();
        let Some(Some(image)) = image_cache.get(&image_id) else {
            log::warn!(
                "Stopping at missing image for guild {} card {}",
                sub.guild_id,
                pending.card.name()
            );
            break;
        };

        // Sent one card at a time (never batched into one message) so each
        // card can have its own Discord thread started on it. Sends for a
        // single subscription stay sequential — this function is never
        // called concurrently with itself for the same subscription — so
        // Discord's per-webhook rate limit is naturally respected without
        // any extra pacing here. Different subscriptions *are* delivered
        // concurrently (see `run`), which is safe since each targets an
        // entirely different webhook.
        match sender
            .send(sub.id, &sub.token, &pending.card, image.clone())
            .await
        {
            Ok(()) => {
                last_queue_id = Some(pending.queue_id);
            }
            Err(e) => {
                log::warn!(
                    "Failed to deliver card to guild {} channel {}: {e}",
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
                // Retried next poll.
                break;
            }
        }
    }

    if let Some(last_queue_id) = last_queue_id {
        repo.ack(sub.guild_id, sub.channel_id, last_queue_id).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::images::MockImageStore;
    use crate::ports::spoilers::{MockSpoilerSender, SpoilerError};
    use cards_sdk::{ChannelId, GuildId, MockSpoilerQueue, SubscriptionId};
    use mockall::predicate::eq;

    fn test_card(name: &str) -> contracts::card::Card {
        let id = Uuid::new_v4();
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
    async fn acks_up_to_the_last_card_after_every_card_sends_successfully() {
        let mut repo = MockSpoilerQueue::new();
        let mut sender = MockSpoilerSender::new();
        let images = always_returns_image_bytes();

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| vec![test_subscription()]);
        repo.expect_pending_cards_since()
            .with(eq(0i64))
            .times(1)
            .return_once(|_| {
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
        // Each card is its own message — two separate sends, not one batch.
        sender.expect_send().times(2).returning(|_, _, _, _| Ok(()));
        repo.expect_ack()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(2u64)), eq(2i64))
            .times(1)
            .return_const(());
        repo.expect_prune_queue().times(1).return_const(());

        run(&repo, &images, &sender).await;
    }

    #[tokio::test]
    async fn acks_only_the_cards_sent_before_a_mid_batch_failure() {
        let mut repo = MockSpoilerQueue::new();
        let mut sender = MockSpoilerSender::new();
        let images = always_returns_image_bytes();

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| vec![test_subscription()]);
        repo.expect_pending_cards_since()
            .with(eq(0i64))
            .times(1)
            .return_once(|_| {
                vec![
                    PendingCard {
                        queue_id: 1,
                        card: test_card("Card A"),
                    },
                    PendingCard {
                        queue_id: 2,
                        card: test_card("Card B"),
                    },
                    // Must never be attempted — the loop stops after card 2
                    // fails, so this card is never sent.
                    PendingCard {
                        queue_id: 3,
                        card: test_card("Card C"),
                    },
                ]
            });
        sender
            .expect_send()
            .withf(|_, _, card, _| card.name() == "Card A")
            .times(1)
            .returning(|_, _, _, _| Ok(()));
        sender
            .expect_send()
            .withf(|_, _, card, _| card.name() == "Card B")
            .times(1)
            .returning(|_, _, _, _| Err(SpoilerError::Status(403)));
        repo.expect_ack()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(2u64)), eq(1i64))
            .times(1)
            .return_const(());
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
        repo.expect_pending_cards_since()
            .with(eq(0i64))
            .times(1)
            .return_once(|_| {
                vec![PendingCard {
                    queue_id: 1,
                    card: test_card("Card A"),
                }]
            });
        sender
            .expect_send()
            .times(1)
            .returning(|_, _, _, _| Err(SpoilerError::Status(403)));
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
        repo.expect_pending_cards_since()
            .with(eq(0i64))
            .times(1)
            .return_once(|_| {
                vec![PendingCard {
                    queue_id: 1,
                    card: test_card("Card A"),
                }]
            });
        sender
            .expect_send()
            .times(1)
            .returning(|_, _, _, _| Err(SpoilerError::Status(404)));
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
        repo.expect_pending_cards_since()
            .with(eq(0i64))
            .times(1)
            .return_once(|_| {
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
            .returning(|_, _, _, _| Err(SpoilerError::Status(503)));
        repo.expect_ack().times(0);
        repo.expect_record_failure().times(0);
        repo.expect_delete_subscription().times(0);
        repo.expect_prune_queue().times(1).return_const(());

        run(&repo, &images, &sender).await;
    }

    #[tokio::test]
    async fn stops_at_the_first_missing_image_and_acks_only_the_cards_before_it() {
        let mut repo = MockSpoilerQueue::new();
        let mut sender = MockSpoilerSender::new();
        let mut images = MockImageStore::new();

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| vec![test_subscription()]);
        repo.expect_pending_cards_since()
            .with(eq(0i64))
            .times(1)
            .return_once(|_| {
                vec![
                    PendingCard {
                        queue_id: 1,
                        card: test_card("Card A"),
                    },
                    PendingCard {
                        queue_id: 2,
                        card: test_card("Missing Image"),
                    },
                    // Image resolution happens up front for the whole
                    // window (not lazily stopping at the first gap, so it
                    // can stay safe to run concurrently across
                    // subscriptions) — so this card's image *is* still
                    // resolved, it's just never delivered, since delivery
                    // itself still stops at the gap left by card 2.
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
        images
            .expect_fetch()
            .times(1)
            .returning(|_| Ok(vec![9, 9, 9]));
        // Only card 1 is ever sent — delivery still stops at the gap.
        sender.expect_send().times(1).returning(|_, _, _, _| Ok(()));
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
        repo.expect_pending_cards_since()
            .with(eq(0i64))
            .times(1)
            .return_once(|_| {
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

    #[tokio::test]
    async fn shares_one_query_and_one_image_fetch_across_multiple_subscriptions() {
        let mut repo = MockSpoilerQueue::new();
        let mut sender = MockSpoilerSender::new();
        let mut images = MockImageStore::new();

        let sub_a = test_subscription(); // guild 1, channel 2, cursor 0
        let sub_b = Subscription {
            guild_id: GuildId::from(1u64),
            channel_id: ChannelId::from(3u64),
            id: SubscriptionId::from(4u64),
            token: "tok-b".to_string(),
            cursor: 1, // already saw card 1, only wants card 2
        };

        repo.expect_subscriptions_with_pending()
            .times(1)
            .return_once(|| vec![sub_a, sub_b]);
        // Called exactly once total, with the *minimum* cursor across both
        // subscriptions (0), even though there are two subscriptions.
        repo.expect_pending_cards_since()
            .with(eq(0i64))
            .times(1)
            .return_once(|_| {
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
        // Card B is needed by both subscriptions but must only be fetched
        // from disk once — the cache serves the second subscriber's copy.
        images
            .expect_fetch()
            .times(2)
            .returning(|_| Ok(vec![9, 9, 9]));

        // sub_a sends both cards as separate messages; sub_b sends only card B.
        sender
            .expect_send()
            .withf(|sub_id, _, card, _| {
                sub_id == &SubscriptionId::from(3u64) && card.name() == "Card A"
            })
            .times(1)
            .returning(|_, _, _, _| Ok(()));
        sender
            .expect_send()
            .withf(|sub_id, _, card, _| {
                sub_id == &SubscriptionId::from(3u64) && card.name() == "Card B"
            })
            .times(1)
            .returning(|_, _, _, _| Ok(()));
        sender
            .expect_send()
            .withf(|sub_id, _, card, _| {
                sub_id == &SubscriptionId::from(4u64) && card.name() == "Card B"
            })
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        repo.expect_ack()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(2u64)), eq(2i64))
            .times(1)
            .return_const(());
        repo.expect_ack()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(3u64)), eq(2i64))
            .times(1)
            .return_const(());
        repo.expect_prune_queue().times(1).return_const(());

        run(&repo, &images, &sender).await;
    }
}
