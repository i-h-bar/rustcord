use crate::ports::drivers::client::MessageInteraction;
use crate::ports::services::spoiler_subscription::{SpoilerSubscription, Subscription};
use cards_sdk::{ChannelId, GuildId, SpoilerQueue};

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
                .reply("Spoiler notifications enabled for this channel.".to_string())
                .await;
        }
        Err(e) => {
            log::warn!("Failed to create spoiler webhook for guild {guild_id}: {e}");
            let _ = interaction
                .reply(
                    "Couldn't create a webhook in that channel — check my permissions.".to_string(),
                )
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
        interaction.expect_reply().times(1).returning(|_| Ok(()));

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
        let mut sub = MockSpoilerSubscription::new();
        let mut storage = MockSpoilerQueue::new();
        let mut interaction = MockMessageInteraction::new();

        sub.expect_create_subscription().times(1).returning(|_| {
            Err(crate::ports::services::spoiler_subscription::SpoilerSubError::new("no perms"))
        });
        storage.expect_create_subscription().times(0);
        interaction.expect_reply().times(1).returning(|_| Ok(()));

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
