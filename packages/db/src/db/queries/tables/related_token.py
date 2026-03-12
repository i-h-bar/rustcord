INSERT = "INSERT INTO related_token (id, card_id, token_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING;"

TRUNCATE = "TRUNCATE TABLE related_token RESTART IDENTITY;"
