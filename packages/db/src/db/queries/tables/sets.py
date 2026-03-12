INSERT = "INSERT INTO set (id, name, normalised_name, abbreviation) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING;"
