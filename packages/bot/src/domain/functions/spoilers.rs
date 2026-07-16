use crate::ports::drivers::client::MessageInteraction;
use crate::ports::services::spoiler_subscription::{SpoilerSubscription, Subscription};
use cards_sdk::{ChannelId, GuildId, SpoilerQueue};
use std::env;

const ISSUES_URL: &str = "https://github.com/i-h-bar/rustcord/issues";

/// URL an admin can visit to re-run the bot's `OAuth2` authorize flow for
/// their server and grant any permissions it's currently missing (e.g.
/// `Manage Webhooks`), without needing to kick and re-invite it. Discord
/// re-derives the bot's permissions from the requested scope each time this
/// is run against a guild it's already in. `permissions` is the existing
/// invite's `277025507328` with `MANAGE_WEBHOOKS` (bit 29, `536870912`)
/// added on top.
///
/// # Panics
/// Panics if `APPLICATION_ID` isn't set.
fn reauthorize_url() -> String {
    let app_id = env::var("APPLICATION_ID").expect("APPLICATION_ID wasn't in env vars");
    format!(
        "https://discord.com/oauth2/authorize?client_id={app_id}&permissions=277562378240&integration_type=0&scope=bot"
    )
}

pub async fn subscribe<S, Sub, I>(
    storage: &S,
    sub: &Sub,
    interaction: &I,
    guild_id: GuildId,
    channel_id: ChannelId,
) where
    S: SpoilerQueue,
    Sub: SpoilerSubscription,
    I: MessageInteraction,
{
    if storage
        .subscription_id(guild_id, channel_id)
        .await
        .is_some()
    {
        let _ = interaction
            .reply_ephemeral(format!(
                "Spoiler notifications are already enabled for <#{}>.",
                u64::from(channel_id)
            ))
            .await;
        return;
    }

    match sub.create_subscription(channel_id).await {
        Ok(Subscription { id, token }) => {
            storage
                .create_subscription(guild_id, channel_id, id, &token)
                .await;
            let _ = interaction
                .reply_ephemeral(format!(
                    "Spoiler notifications enabled for <#{}>. This feature is in beta — \
                     if anything looks wrong, please let us know here: {ISSUES_URL}",
                    u64::from(channel_id)
                ))
                .await;
        }
        Err(e) => {
            log::warn!("Failed to create spoiler webhook for guild {guild_id}: {e}");
            let _ = interaction
                .reply_ephemeral(format!(
                    "Couldn't create a webhook in that channel — I'm missing permissions. \
                     Ask a server admin to re-authorize me here to grant what's missing: {}",
                    reauthorize_url()
                ))
                .await;
        }
    }
}

pub async fn unsubscribe<S, Sub, I>(
    storage: &S,
    sub: &Sub,
    interaction: &I,
    guild_id: GuildId,
    channel_id: ChannelId,
) where
    S: SpoilerQueue,
    Sub: SpoilerSubscription,
    I: MessageInteraction,
{
    let Some(sub_id) = storage.subscription_id(guild_id, channel_id).await else {
        let _ = interaction
            .reply_ephemeral(format!(
                "Spoiler notifications disabled for <#{}>.",
                u64::from(channel_id)
            ))
            .await;
        return;
    };

    // The DB record is only removed once Discord confirms the webhook is
    // actually gone (deleted just now, or already gone with a 404) — a
    // Discord-side outage or other transient error must leave the
    // subscription intact, or the webhook would be orphaned with nothing
    // left tracking it for a retry.
    match sub.delete_subscription(sub_id).await {
        Ok(()) => {
            storage.delete_subscription(guild_id, channel_id).await;
        }
        Err(e) if e.is_not_found() => {
            log::info!(
                "Webhook {sub_id} for guild {guild_id} channel {channel_id} was already gone; removing subscription record"
            );
            storage.delete_subscription(guild_id, channel_id).await;
        }
        Err(e) => {
            log::warn!(
                "Failed to delete Discord webhook {sub_id} for guild {guild_id} channel {channel_id}, keeping subscription: {e}"
            );
            let _ = interaction
                .reply_ephemeral(format!(
                    "Couldn't reach Discord to remove the webhook for <#{}> — nothing changed, please try again shortly.",
                    u64::from(channel_id)
                ))
                .await;
            return;
        }
    }

    let _ = interaction
        .reply_ephemeral(format!(
            "Spoiler notifications disabled for <#{}>.",
            u64::from(channel_id)
        ))
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::drivers::client::MockMessageInteraction;
    use crate::ports::services::spoiler_subscription::MockSpoilerSubscription;
    use cards_sdk::{MockSpoilerQueue, SubscriptionId};
    use mockall::predicate::eq;

    #[tokio::test]
    async fn subscribe_creates_webhook_then_stores_subscription() {
        let mut sub = MockSpoilerSubscription::new();
        let mut storage = MockSpoilerQueue::new();
        let mut interaction = MockMessageInteraction::new();

        storage
            .expect_subscription_id()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(42u64)))
            .times(1)
            .return_const(None);
        sub.expect_create_subscription()
            .with(eq(ChannelId::from(42u64)))
            .times(1)
            .returning(|_| {
                Ok(Subscription {
                    id: SubscriptionId::from(99u64),
                    token: "tok".to_string(),
                })
            });
        storage
            .expect_create_subscription()
            .with(
                eq(GuildId::from(1u64)),
                eq(ChannelId::from(42u64)),
                eq(SubscriptionId::from(99u64)),
                eq("tok"),
            )
            .times(1)
            .return_const(());
        interaction
            .expect_reply_ephemeral()
            .times(1)
            .returning(|_| Ok(()));

        subscribe(
            &storage,
            &sub,
            &interaction,
            GuildId::from(1u64),
            ChannelId::from(42u64),
        )
        .await;
    }

    #[tokio::test]
    async fn subscribe_does_not_create_a_webhook_when_the_channel_is_already_subscribed() {
        let sub = MockSpoilerSubscription::new();
        let mut storage = MockSpoilerQueue::new();
        let mut interaction = MockMessageInteraction::new();

        storage
            .expect_subscription_id()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(42u64)))
            .times(1)
            .return_const(Some(SubscriptionId::from(99u64)));
        // `sub`/`storage` have no `create_subscription` expectations at all
        // — a second webhook must never be created for an already-subscribed
        // channel.
        interaction
            .expect_reply_ephemeral()
            .times(1)
            .returning(|_| Ok(()));

        subscribe(
            &storage,
            &sub,
            &interaction,
            GuildId::from(1u64),
            ChannelId::from(42u64),
        )
        .await;
    }

    #[tokio::test]
    async fn subscribe_does_not_store_subscription_when_webhook_creation_fails() {
        // SAFETY: `reauthorize_url` reads this; tests run single-threaded
        // enough within this module that a fixed test value is safe here.
        unsafe { env::set_var("APPLICATION_ID", "123") };

        let mut sub = MockSpoilerSubscription::new();
        let mut storage = MockSpoilerQueue::new();
        let mut interaction = MockMessageInteraction::new();

        storage.expect_subscription_id().times(1).return_const(None);
        sub.expect_create_subscription().times(1).returning(|_| {
            Err(crate::ports::services::spoiler_subscription::SpoilerSubError::new("no perms"))
        });
        storage.expect_create_subscription().times(0);
        interaction
            .expect_reply_ephemeral()
            .times(1)
            .returning(|_| Ok(()));

        subscribe(
            &storage,
            &sub,
            &interaction,
            GuildId::from(1u64),
            ChannelId::from(42u64),
        )
        .await;
    }

    #[tokio::test]
    async fn unsubscribe_deletes_subscription_and_webhook() {
        let mut storage = MockSpoilerQueue::new();
        let mut sub = MockSpoilerSubscription::new();
        let mut interaction = MockMessageInteraction::new();

        storage
            .expect_subscription_id()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(42u64)))
            .times(1)
            .return_const(Some(SubscriptionId::from(99u64)));
        sub.expect_delete_subscription()
            .with(eq(SubscriptionId::from(99u64)))
            .times(1)
            .returning(|_| Ok(()));
        storage
            .expect_delete_subscription()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(42u64)))
            .times(1)
            .return_once(|_, _| Some(SubscriptionId::from(99u64)));
        interaction
            .expect_reply_ephemeral()
            .times(1)
            .returning(|_| Ok(()));

        unsubscribe(
            &storage,
            &sub,
            &interaction,
            GuildId::from(1u64),
            ChannelId::from(42u64),
        )
        .await;
    }

    #[tokio::test]
    async fn unsubscribe_is_a_no_op_when_already_unsubscribed() {
        let mut storage = MockSpoilerQueue::new();
        let sub = MockSpoilerSubscription::new();
        let mut interaction = MockMessageInteraction::new();

        storage
            .expect_subscription_id()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(42u64)))
            .times(1)
            .return_const(None);
        // `sub`/`storage` have no `delete_subscription` expectations at all
        // — nothing to delete on either side when there was no subscription.
        interaction
            .expect_reply_ephemeral()
            .times(1)
            .returning(|_| Ok(()));

        unsubscribe(
            &storage,
            &sub,
            &interaction,
            GuildId::from(1u64),
            ChannelId::from(42u64),
        )
        .await;
    }

    #[tokio::test]
    async fn unsubscribe_removes_the_record_when_discord_reports_the_webhook_already_gone() {
        let mut storage = MockSpoilerQueue::new();
        let mut sub = MockSpoilerSubscription::new();
        let mut interaction = MockMessageInteraction::new();

        storage
            .expect_subscription_id()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(42u64)))
            .times(1)
            .return_const(Some(SubscriptionId::from(99u64)));
        sub.expect_delete_subscription()
            .with(eq(SubscriptionId::from(99u64)))
            .times(1)
            .returning(|_| {
                Err(
                    crate::ports::services::spoiler_subscription::SpoilerSubError::with_status(
                        "not found",
                        404,
                    ),
                )
            });
        // A 404 means the webhook is already gone — safe to drop our record.
        storage
            .expect_delete_subscription()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(42u64)))
            .times(1)
            .return_once(|_, _| Some(SubscriptionId::from(99u64)));
        interaction
            .expect_reply_ephemeral()
            .times(1)
            .returning(|_| Ok(()));

        unsubscribe(
            &storage,
            &sub,
            &interaction,
            GuildId::from(1u64),
            ChannelId::from(42u64),
        )
        .await;
    }

    #[tokio::test]
    async fn unsubscribe_keeps_the_record_when_discord_is_unreachable() {
        let mut storage = MockSpoilerQueue::new();
        let mut sub = MockSpoilerSubscription::new();
        let mut interaction = MockMessageInteraction::new();

        storage
            .expect_subscription_id()
            .with(eq(GuildId::from(1u64)), eq(ChannelId::from(42u64)))
            .times(1)
            .return_const(Some(SubscriptionId::from(99u64)));
        sub.expect_delete_subscription()
            .with(eq(SubscriptionId::from(99u64)))
            .times(1)
            .returning(|_| {
                Err(
                    crate::ports::services::spoiler_subscription::SpoilerSubError::new(
                        "connection reset",
                    ),
                )
            });
        // Must NOT be called — a non-404 failure must leave the subscription
        // record in place so it can be retried later.
        storage.expect_delete_subscription().times(0);
        interaction
            .expect_reply_ephemeral()
            .times(1)
            .returning(|_| Ok(()));

        unsubscribe(
            &storage,
            &sub,
            &interaction,
            GuildId::from(1u64),
            ChannelId::from(42u64),
        )
        .await;
    }
}
