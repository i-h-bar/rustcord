mod discord;

use crate::adapters::services::spoiler_subscription::discord::DiscordWebhookRegistrar;
use crate::ports::services::spoiler_subscription::SpoilerSubscription;

#[must_use]
pub fn init_spoiler_subscription() -> impl SpoilerSubscription {
    DiscordWebhookRegistrar::create()
}
