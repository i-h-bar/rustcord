-- Fix cmc: INTEGER cannot hold f64 values (Scryfall CMC can be fractional)
ALTER TABLE rule ALTER COLUMN cmc TYPE DOUBLE PRECISION;

-- Fix updated_time: TIME WITH TIME ZONE has no date component,
-- but the application writes full timestamps (OffsetDateTime).
-- Existing values are nulled out — the next sync repopulates all prices.
ALTER TABLE price ALTER COLUMN updated_time TYPE TIMESTAMP WITH TIME ZONE USING NULL;
