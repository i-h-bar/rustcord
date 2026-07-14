CREATE TABLE IF NOT EXISTS spoiler_queue (
    id BIGSERIAL PRIMARY KEY,
    card_id UUID NOT NULL REFERENCES card(id),
    detected_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS spoiler_queue_card_id_idx ON spoiler_queue (card_id);

CREATE TABLE IF NOT EXISTS spoiler_subscription (
    guild_id BIGINT PRIMARY KEY,
    channel_id BIGINT NOT NULL,
    webhook_id BIGINT NOT NULL,
    webhook_token TEXT NOT NULL,
    cursor BIGINT NOT NULL,
    consecutive_failures INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
