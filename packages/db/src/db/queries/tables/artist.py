INSERT = "INSERT into artist (id, name, normalised_name) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING;"
