pub const FUZZY_SEARCH_DISTINCT_CARDS: &str = r#"
select *
from distinct_cards
where word_similarity(front_normalised_name, $1) > 0.50;
"#;


pub const NORMALISED_SET_NAME: &str = r#"select normalised_name from set where abbreviation = $1"#;