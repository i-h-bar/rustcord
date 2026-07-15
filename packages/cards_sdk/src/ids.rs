use std::fmt;
use uuid::Uuid;

macro_rules! sub_id {
    ($name:ident) => {
        /// Invertible newtype around a Discord snowflake ID, stored as a
        /// `UUID` in Postgres rather than a signed `BIGINT` — Postgres has
        /// no unsigned integer type, and mapping a `u64` snowflake straight
        /// into `BIGINT` meant every read/write needed a sign-reinterpreting
        /// cast, with a real risk of the two directions drifting out of
        /// sync (`as i64`/`cast_signed` on write vs. checked `try_from` on
        /// read, which silently rejects a value the write side happily
        /// stored). A `UUID` sidesteps signedness entirely and decouples
        /// this crate's schema from Discord's specific ID representation,
        /// so a future non-Discord subscription source doesn't have to be
        /// shoehorned into a `u64`.
        ///
        /// The snowflake is embedded verbatim in the low 8 bytes with the
        /// high 8 bytes zeroed — deliberately *not* an RFC-4122-compliant
        /// UUID (no version/variant bits set), just a 128-bit container
        /// wide enough to hold a 64-bit platform-native ID without sign
        /// ambiguity. `From<u64>` and `From<$name> for u64` (so `.into()`
        /// works in both directions) are exact inverses of each other
        /// across the full `u64` range.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type)]
        #[sqlx(transparent)]
        pub struct $name(Uuid);

        impl $name {
            #[must_use]
            pub fn uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<u64> for $name {
            fn from(id: u64) -> Self {
                let mut bytes = [0u8; 16];
                bytes[8..16].copy_from_slice(&id.to_be_bytes());
                Self(Uuid::from_bytes(bytes))
            }
        }

        impl From<$name> for u64 {
            fn from(id: $name) -> u64 {
                let bytes = id.0.as_bytes();
                u64::from_be_bytes(bytes[8..16].try_into().expect("slice is exactly 8 bytes"))
            }
        }

        /// Shows the Discord snowflake first (the externally meaningful
        /// value — what shows up in bug reports, Discord's own UI, other
        /// logs) followed by the stored `UUID` in parentheses, so a raw
        /// `SELECT * FROM spoiler_subscription` dump can still be matched
        /// back to a log line without running a decode query.
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{} ({})", u64::from(*self), self.0)
            }
        }
    };
}

sub_id!(GuildId);
sub_id!(ChannelId);
sub_id!(SubscriptionId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guild_id_round_trips_small_values() {
        let id = GuildId::from(42u64);
        assert_eq!(u64::from(id), 42);
    }

    #[test]
    fn channel_id_round_trips_the_full_u64_range() {
        for raw in [0u64, 1, u64::MAX / 2, u64::MAX - 1, u64::MAX] {
            let id: ChannelId = raw.into();
            let back: u64 = id.into();
            assert_eq!(back, raw);
        }
    }

    #[test]
    fn subscription_id_embeds_in_the_low_bytes_with_high_bytes_zeroed() {
        let id = SubscriptionId::from(1u64);
        let bytes = id.uuid().into_bytes();
        assert_eq!(&bytes[0..8], &[0u8; 8]);
        assert_eq!(&bytes[8..16], &1u64.to_be_bytes());
    }

    #[test]
    fn display_shows_the_snowflake_and_the_stored_uuid() {
        let id = GuildId::from(1u64);
        assert_eq!(id.to_string(), "1 (00000000-0000-0000-0000-000000000001)");
    }

    #[test]
    fn distinct_ids_of_the_same_value_are_equal() {
        assert_eq!(GuildId::from(7u64), GuildId::from(7u64));
        assert_ne!(GuildId::from(7u64).uuid(), GuildId::from(8u64).uuid());
    }
}
