UPSERT = """
  INSERT INTO image (id, scryfall_url)
  VALUES ($1, $2)
  ON CONFLICT (id) DO UPDATE
    SET scryfall_url = EXCLUDED.scryfall_url
  WHERE image.scryfall_url IS DISTINCT FROM EXCLUDED.scryfall_url;
         """
