pub const FUZZY_SEARCH_DISTINCT_CARDS: &str = r#"
select  front_name,
        front_normalised_name,
        front_scryfall_url,
        front_image_id,
        front_mana_cost,
        front_colour_identity,
        front_power,
        front_toughness,
        front_loyalty,
        front_defence,
        front_type_line,
        front_oracle_text,
        back_name,
        back_scryfall_url,
        back_image_id,
        back_mana_cost,
        back_colour_identity,
        back_power,
        back_toughness,
        back_loyalty,
        back_defence,
        back_type_line,
        back_oracle_text
    from distinct_cards
where word_similarity(front_normalised_name, $1) > 0.50;
"#;

pub const FUZZY_SEARCH_SET_NAME: &str = r#"
select array_agg(normalised_name)
    from set
where word_similarity(normalised_name, $1) > 0.50
"#;

pub const FUZZY_SEARCH_ARTIST: &str = r#"
select array_agg(normalised_name)
    from artist
where word_similarity(normalised_name, $1) > 0.50
"#;

pub const NORMALISED_SET_NAME: &str = r#"select normalised_name from set where abbreviation = $1"#;
