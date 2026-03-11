UPSERT = """
         INSERT INTO card
         (id,
          oracle_id,
          name,
          normalised_name,
          scryfall_url,
          flavour_text,
          release_date,
          reserved,
          rarity,
          artist_id,
          image_id,
          illustration_id,
          set_id,
          backside_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
         ON CONFLICT (id) DO UPDATE SET
             normalised_name = EXCLUDED.normalised_name,
             scryfall_url    = EXCLUDED.scryfall_url,
             reserved        = EXCLUDED.reserved,
             oracle_id       = EXCLUDED.oracle_id
         WHERE (card.normalised_name IS DISTINCT FROM EXCLUDED.normalised_name OR
                card.scryfall_url    IS DISTINCT FROM EXCLUDED.scryfall_url    OR
                card.reserved        IS DISTINCT FROM EXCLUDED.reserved        OR
                card.oracle_id       IS DISTINCT FROM EXCLUDED.oracle_id
                );
         """
