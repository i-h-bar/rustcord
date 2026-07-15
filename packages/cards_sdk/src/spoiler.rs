use crate::ids::{ChannelId, GuildId, SubscriptionId};
use contracts::card::Card;

pub struct Subscription {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub webhook_id: SubscriptionId,
    pub webhook_token: String,
    pub cursor: i64,
}

/// A single queued card ready for delivery to a guild, paired with the
/// queue row it came from (for acking) and the card image's own
/// `scryfall_url` (distinct from `Card::url`, which is the Scryfall page
/// link) — `notifier` sets the embed image directly to this URL rather than
/// attaching a local file, to avoid mounting the card-image volume into a
/// scale-to-zero pod.
pub struct PendingCard {
    pub queue_id: i64,
    pub card: Card,
    pub image_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn subscription_fields_are_accessible() {
        let sub = Subscription {
            guild_id: GuildId::from(1u64),
            channel_id: ChannelId::from(2u64),
            webhook_id: SubscriptionId::from(3u64),
            webhook_token: "token".to_string(),
            cursor: 0,
        };
        assert_eq!(u64::from(sub.guild_id), 1);
    }

    #[test]
    fn pending_card_fields_are_accessible() {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let card = Card::new(
            id,
            "Lightning Bolt".to_string(),
            "lightning bolt".to_string(),
            id,
            "https://scryfall.com/card/lea/1".to_string(),
            id,
            None,
            "{R}".to_string(),
            vec!["R".to_string()],
            None,
            None,
            None,
            None,
            "Instant".to_string(),
            "Deal 3 damage".to_string(),
            None,
            "Artist".to_string(),
            "Alpha".to_string(),
            "LEA".to_string(),
            time::Date::from_calendar_date(1993, time::Month::August, 5).unwrap(),
        );
        let pending = PendingCard {
            queue_id: 5,
            card,
            image_url: "https://img".to_string(),
        };
        assert_eq!(pending.queue_id, 5);
    }
}
