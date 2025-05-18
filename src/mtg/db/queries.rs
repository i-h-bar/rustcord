pub const FUZZY_SEARCH_DISTINCT_CARDS: &str = r#"
select * from distinct_cards
where word_similarity(front_normalised_name, $1) > 0.50;
"#;

pub const FUZZY_SEARCH_SET_NAME: &str = r#"
select array_agg(normalised_name)
    from set
where word_similarity(normalised_name, $1) > 0.25
"#;

pub const FUZZY_SEARCH_ARTIST: &str = r#"
select array_agg(normalised_name)
    from artist
where word_similarity(normalised_name, $1) > 0.50
"#;

pub const NORMALISED_SET_NAME: &str = r#"select normalised_name from set where abbreviation = $1"#;

pub const RANDOM_CARD_FROM_DISTINCT: &str = r#"
select * from distinct_cards order by random() limit 1;
"#;
