CREATE TABLE IF NOT EXISTS spoiler_queue (
    id BIGSERIAL PRIMARY KEY,
    card_id UUID NOT NULL REFERENCES card(id),
    detected_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS spoiler_queue_card_id_idx ON spoiler_queue (card_id);

CREATE TABLE IF NOT EXISTS spoiler_subscription (
    -- guild_id/channel_id/webhook_id are Discord snowflakes (u64) embedded
    -- into a UUID's low 8 bytes (high 8 bytes zeroed) rather than stored as
    -- BIGINT — Postgres has no unsigned integer type, and BIGINT would need
    -- a sign-reinterpreting cast on every read/write. See
    -- cards_sdk::ids for the Rust-side newtypes and the round-trip logic.
    guild_id UUID PRIMARY KEY,
    channel_id UUID NOT NULL,
    webhook_id UUID NOT NULL,
    webhook_token TEXT NOT NULL,
    cursor BIGINT NOT NULL,
    consecutive_failures INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
