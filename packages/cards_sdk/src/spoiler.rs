use crate::ids::{ChannelId, GuildId, SubscriptionId};
use contracts::card::Card;

pub struct Subscription {
    pub id: SubscriptionId,
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub token: String,
    pub cursor: i64,
}

/// A single queued card ready for delivery to a guild, paired with the
/// queue row it came from (for acking). The image itself is read from the
/// local `IMAGES_DIR` cache by `notifier` (via `card.image_id()`), the same
/// as `bot` does, rather than pointed at by a remote URL.
pub struct PendingCard {
    pub queue_id: i64,
    pub card: Card,
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
            id: SubscriptionId::from(3u64),
            token: "token".to_string(),
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
        let pending = PendingCard { queue_id: 5, card };
        assert_eq!(pending.queue_id, 5);
    }
}
