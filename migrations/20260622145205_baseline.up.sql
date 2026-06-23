-- Baseline migration: captures the full schema as of the alembic migration chain
-- (85fb9545aec4 → 6c6c92fc7e9b → fa68030fe594 → c9bb76b8e5ee)
--
-- Idempotent: safe to run on an existing production DB that already has the schema.

CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE IF NOT EXISTS artist (
    id UUID PRIMARY KEY,
    name TEXT,
    normalised_name TEXT
);

CREATE TABLE IF NOT EXISTS image (
    id UUID PRIMARY KEY,
    scryfall_url TEXT
);

CREATE TABLE IF NOT EXISTS illustration (
    id UUID PRIMARY KEY,
    scryfall_url TEXT
);

CREATE TABLE IF NOT EXISTS legality (
    id UUID PRIMARY KEY,
    alchemy TEXT,
    brawl TEXT,
    commander TEXT,
    duel TEXT,
    future TEXT,
    gladiator TEXT,
    historic TEXT,
    legacy TEXT,
    modern TEXT,
    oathbreaker TEXT,
    oldschool TEXT,
    pauper TEXT,
    paupercommander TEXT,
    penny TEXT,
    pioneer TEXT,
    predh TEXT,
    premodern TEXT,
    standard TEXT,
    standardbrawl TEXT,
    timeless TEXT,
    vintage TEXT,
    game_changer BOOLEAN
);

CREATE TABLE IF NOT EXISTS rule (
    id UUID PRIMARY KEY,
    colour_identity CHAR(1)[],
    mana_cost TEXT,
    cmc INTEGER,
    power TEXT,
    toughness TEXT,
    loyalty TEXT,
    defence TEXT,
    type_line TEXT,
    oracle_text TEXT,
    colours CHAR(1)[],
    keywords TEXT[],
    produced_mana CHAR(1)[],
    rulings_url TEXT
);

CREATE TABLE IF NOT EXISTS set (
    id UUID PRIMARY KEY,
    name TEXT,
    normalised_name TEXT,
    abbreviation TEXT
);

CREATE TABLE IF NOT EXISTS card (
    id UUID PRIMARY KEY,
    oracle_id UUID,
    name TEXT,
    normalised_name TEXT,
    scryfall_url TEXT,
    flavour_text TEXT,
    release_date DATE,
    reserved BOOLEAN,
    rarity TEXT,
    artist_id UUID,
    image_id UUID,
    illustration_id UUID,
    set_id UUID,
    backside_id UUID,
    FOREIGN KEY (artist_id) REFERENCES artist (id),
    FOREIGN KEY (illustration_id) REFERENCES illustration (id),
    FOREIGN KEY (image_id) REFERENCES image (id),
    FOREIGN KEY (oracle_id) REFERENCES legality (id),
    FOREIGN KEY (oracle_id) REFERENCES rule (id),
    FOREIGN KEY (set_id) REFERENCES set (id)
);

CREATE TABLE IF NOT EXISTS combo (
    id UUID PRIMARY KEY,
    card_id UUID,
    combo_card_id UUID,
    FOREIGN KEY (card_id) REFERENCES card (id),
    FOREIGN KEY (combo_card_id) REFERENCES card (id)
);

CREATE TABLE IF NOT EXISTS price (
    id UUID PRIMARY KEY,
    usd NUMERIC(10, 2),
    usd_foil NUMERIC(10, 2),
    usd_etched NUMERIC(10, 2),
    euro NUMERIC(10, 2),
    euro_foil NUMERIC(10, 2),
    tix NUMERIC(10, 2),
    updated_time TIME WITH TIME ZONE,
    FOREIGN KEY (id) REFERENCES card (id)
);

CREATE TABLE IF NOT EXISTS related_token (
    id UUID PRIMARY KEY,
    card_id UUID,
    token_id UUID,
    FOREIGN KEY (card_id) REFERENCES card (id),
    FOREIGN KEY (token_id) REFERENCES card (id)
);

CREATE INDEX IF NOT EXISTS idx_gin_card_normalised_name ON card USING gin (normalised_name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_gin_set_normalised_name ON set USING gin (normalised_name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_gin_artist_normalised_name ON artist USING gin (normalised_name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS ix_card_set_id ON card (set_id);
CREATE INDEX IF NOT EXISTS ix_card_oracle_id ON card (oracle_id);

ALTER DATABASE mtg SET pg_trgm.similarity_threshold = 0.3;

DROP TABLE IF EXISTS alembic_version;