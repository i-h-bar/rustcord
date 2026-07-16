CREATE TABLE IF NOT EXISTS spoiler_queue (
    id BIGSERIAL PRIMARY KEY,
    card_id UUID NOT NULL REFERENCES card(id),
    detected_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS spoiler_queue_card_id_idx ON spoiler_queue (card_id);

CREATE TABLE IF NOT EXISTS spoiler_subscription (
    -- guild_id/channel_id/subscription_id are Discord snowflakes (u64)
    -- embedded into a UUID's low 8 bytes (high 8 bytes zeroed) rather than
    -- stored as BIGINT — Postgres has no unsigned integer type, and BIGINT
    -- would need a sign-reinterpreting cast on every read/write. See
    -- cards_sdk::ids for the Rust-side newtypes and the round-trip logic.
    -- `subscription_id`/`subscription_token` are named generically rather
    -- than after Discord's "webhook" concept, since this table is meant to
    -- outlive Discord-specific delivery if a future notification source is
    -- ever added — only the concrete Discord adapter should know these are
    -- backed by a webhook at all.
    guild_id UUID NOT NULL,
    channel_id UUID NOT NULL,
    subscription_id UUID NOT NULL,
    subscription_token TEXT NOT NULL,
    cursor BIGINT NOT NULL,
    consecutive_failures INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- A guild may subscribe multiple channels independently, each with its
    -- own subscription/cursor/failure count — guild_id leads the key so
    -- guild-scoped lookups (e.g. deleting every subscription for a guild
    -- the bot has left) still use this index as a prefix.
    PRIMARY KEY (guild_id, channel_id)
);
