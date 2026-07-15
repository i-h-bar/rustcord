use crate::ports::drivers::client::MessageInteraction;
use crate::ports::services::spoiler_subscription::{SpoilerSubscription, Subscription};
use cards_sdk::{ChannelId, GuildId, SpoilerQueue};
use std::env;

const ISSUES_URL: &str = "https://github.com/i-h-bar/rustcord/issues";

/// URL an admin can visit to re-run the bot's OAuth2 authorize flow for
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
    match sub.create_subscription(channel_id).await {
        Ok(Subscription { id, token }) => {
            storage
                .create_subscription(guild_id, channel_id, id, &token)
                .await;
            let _ = interaction
                .reply_ephemeral(format!(
                    "Spoiler notifications enabled for this channel. This feature is in beta — \
                     if anything looks wrong, please let us know here: {ISSUES_URL}"
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

pub async fn unsubscribe<S, Sub, I>(storage: &S, sub: &Sub, interaction: &I, guild_id: GuildId)
where
    S: SpoilerQueue,
    Sub: SpoilerSubscription,
    I: MessageInteraction,
{
    if let Some(sub_id) = storage.delete_subscription(guild_id).await {
        if let Err(e) = sub.delete_subscription(sub_id).await {
            log::warn!("Failed to delete Discord webhook {sub_id} for guild {guild_id}: {e}");
        }
    }
    let _ = interaction
        .reply("Spoiler notifications disabled for this server.".to_string())
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
    async fn subscribe_does_not_store_subscription_when_webhook_creation_fails() {
        // SAFETY: `reauthorize_url` reads this; tests run single-threaded
        // enough within this module that a fixed test value is safe here.
        unsafe { env::set_var("APPLICATION_ID", "123") };

        let mut sub = MockSpoilerSubscription::new();
        let mut storage = MockSpoilerQueue::new();
        let mut interaction = MockMessageInteraction::new();

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
            .expect_delete_subscription()
            .with(eq(GuildId::from(1u64)))
            .times(1)
            .return_once(|_| Some(SubscriptionId::from(99u64)));
        sub.expect_delete_subscription()
            .with(eq(SubscriptionId::from(99u64)))
            .times(1)
            .returning(|_| Ok(()));
        interaction.expect_reply().times(1).returning(|_| Ok(()));

        unsubscribe(&storage, &sub, &interaction, GuildId::from(1u64)).await;
    }

    #[tokio::test]
    async fn unsubscribe_is_a_no_op_on_the_webhook_when_already_unsubscribed() {
        let mut storage = MockSpoilerQueue::new();
        let mut sub = MockSpoilerSubscription::new();
        let mut interaction = MockMessageInteraction::new();

        storage
            .expect_delete_subscription()
            .with(eq(GuildId::from(1u64)))
            .times(1)
            .return_once(|_| None);
        sub.expect_delete_subscription().times(0);
        interaction.expect_reply().times(1).returning(|_| Ok(()));

        unsubscribe(&storage, &sub, &interaction, GuildId::from(1u64)).await;
    }
}
