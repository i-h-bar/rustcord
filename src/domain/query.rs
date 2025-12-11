use crate::domain::utils;
use regex::Captures;

#[cfg_attr(test, derive(Clone))]
pub struct QueryParams {
    artist: Option<String>,
    name: String,
    set_code: Option<String>,
    set_name: Option<String>,
}

impl QueryParams {
    #[must_use]
    pub fn new(
        artist: Option<String>,
        name: String,
        set_code: Option<String>,
        set_name: Option<String>,
    ) -> Self {
        Self {
            artist,
            name,
            set_code,
            set_name,
        }
    }

    #[must_use]
    pub fn from(capture: &Captures<'_>) -> Option<Self> {
        let raw_name = capture.get(1)?.as_str().trim();
        let name = utils::normalise(raw_name);
        let (set_code, set_name) = match capture.get(4) {
            Some(set) => {
                let set = set.as_str().trim();
                if set.chars().count() < 5 {
                    (Some(utils::normalise(set)), None)
                } else {
                    (None, Some(utils::normalise(set)))
                }
            }
            None => (None, None),
        };

        let artist = capture
            .get(7)
            .map(|artist| utils::normalise(artist.as_str().trim()));

        Some(Self {
            artist,
            name,
            set_code,
            set_name,
        })
    }

    #[cfg(test)]
    pub fn from_test(
        name: String,
        artist: Option<String>,
        set_name: Option<String>,
        set_code: Option<String>,
    ) -> Self {
        Self {
            name,
            artist,
            set_name,
            set_code,
        }
    }

    #[must_use]
    pub fn set_code(&self) -> Option<&String> {
        self.set_code.as_ref()
    }

    #[must_use]
    pub fn set_name(&self) -> Option<&String> {
        self.set_name.as_ref()
    }

    #[must_use]
    pub fn artist(&self) -> Option<&String> {
        self.artist.as_ref()
    }

    #[must_use]
    pub fn name(&self) -> &String {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::utils::REGEX_COLLECTION;

    #[test]
    fn test_query_params_new() {
        let params = QueryParams::new(
            Some("artist name".to_string()),
            "card name".to_string(),
            Some("m11".to_string()),
            None,
        );

        assert_eq!(params.name(), "card name");
        assert_eq!(params.artist(), Some(&"artist name".to_string()));
        assert_eq!(params.set_code(), Some(&"m11".to_string()));
        assert_eq!(params.set_name(), None);
    }

    #[test]
    fn test_query_params_from_simple_card() {
        let text = "[[lightning bolt]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.name(), "lightning bolt");
        assert_eq!(params.artist(), None);
        assert_eq!(params.set_code(), None);
        assert_eq!(params.set_name(), None);
    }

    #[test]
    fn test_query_params_from_card_with_set_code() {
        let text = "[[lightning bolt | set=m11]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.name(), "lightning bolt");
        assert_eq!(params.set_code(), Some(&"m11".to_string()));
        assert_eq!(params.set_name(), None);
        assert_eq!(params.artist(), None);
    }

    #[test]
    fn test_query_params_from_card_with_set_name() {
        let text = "[[lightning bolt | set=bloomburrow commander]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.name(), "lightning bolt");
        assert_eq!(params.set_code(), None);
        assert_eq!(
            params.set_name(),
            Some(&"bloomburrow commander".to_string())
        );
        assert_eq!(params.artist(), None);
    }

    #[test]
    fn test_query_params_set_boundary_4_chars() {
        // 4 chars = set code
        let text = "[[card | set=m10x]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.set_code(), Some(&"m10x".to_string()));
        assert_eq!(params.set_name(), None);
    }

    #[test]
    fn test_query_params_set_boundary_5_chars() {
        // 5 chars = set name
        let text = "[[card | set=tenth]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.set_code(), None);
        assert_eq!(params.set_name(), Some(&"tenth".to_string()));
    }

    #[test]
    fn test_query_params_from_card_with_artist() {
        let text = "[[relentless rats | artist=thomas m baxa]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.name(), "relentless rats");
        assert_eq!(params.artist(), Some(&"thomas m baxa".to_string()));
        assert_eq!(params.set_code(), None);
        assert_eq!(params.set_name(), None);
    }

    #[test]
    fn test_query_params_from_card_with_set_and_artist() {
        let text = "[[gitrog monster | set=soi | artist=jason kang]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.name(), "gitrog monster");
        assert_eq!(params.set_code(), Some(&"soi".to_string()));
        assert_eq!(params.artist(), Some(&"jason kang".to_string()));
    }

    #[test]
    fn test_query_params_with_punctuation() {
        // Normalization should remove punctuation
        let text = "[[Jace, the Mind Sculptor]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.name(), "jace the mind sculptor");
    }

    #[test]
    fn test_query_params_case_normalization() {
        let text = "[[LIGHTNING BOLT]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.name(), "lightning bolt");
    }

    #[test]
    fn test_query_params_with_spaces_around_equals() {
        let text = "[[card | set = m11]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.set_code(), Some(&"m11".to_string()));
    }

    #[test]
    fn test_query_params_with_spaces_around_pipe() {
        let text = "[[card name  |  set=core]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.name(), "card name");
        // "core" is 4 chars, so it's treated as a set code
        assert_eq!(params.set_code(), Some(&"core".to_string()));
        assert_eq!(params.set_name(), None);
    }

    #[test]
    fn test_query_params_multiple_cards_in_message() {
        let text = "I love [[lightning bolt]] and [[counterspell]]";
        let captures: Vec<_> = REGEX_COLLECTION.cards.captures_iter(text).collect();

        assert_eq!(captures.len(), 2);

        let params1 = QueryParams::from(&captures[0]).unwrap();
        let params2 = QueryParams::from(&captures[1]).unwrap();

        assert_eq!(params1.name(), "lightning bolt");
        assert_eq!(params2.name(), "counterspell");
    }

    #[test]
    fn test_query_params_complex_example() {
        let text = "[[the gitrog monster | set=bloomburrow commander]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.name(), "the gitrog monster");
        assert_eq!(
            params.set_name(),
            Some(&"bloomburrow commander".to_string())
        );
    }

    #[test]
    fn test_query_params_from_test_helper() {
        let params = QueryParams::from_test(
            "gitrog monster".to_string(),
            Some("jason kang".to_string()),
            Some("shadows over innistrad".to_string()),
            None,
        );

        assert_eq!(params.name(), "gitrog monster");
        assert_eq!(params.artist(), Some(&"jason kang".to_string()));
        assert_eq!(
            params.set_name(),
            Some(&"shadows over innistrad".to_string())
        );
        assert_eq!(params.set_code(), None);
    }

    #[test]
    fn test_query_params_artist_normalization() {
        let text = "[[card | artist=John Avon]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.artist(), Some(&"john avon".to_string()));
    }

    #[test]
    fn test_query_params_set_normalization() {
        let text = "[[card | set=Core Set 2021]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.set_name(), Some(&"core set 2021".to_string()));
    }

    #[test]
    fn test_invalid_regex_returns_none() {
        let text = "just regular text";
        let captures = REGEX_COLLECTION.cards.captures(text);

        assert!(captures.is_none());
    }

    #[test]
    fn test_query_params_set_no_whitespace() {
        let text = "[[card|set=Core Set 2021]]";
        let captures = REGEX_COLLECTION.cards.captures(text).unwrap();
        let params = QueryParams::from(&captures).unwrap();

        assert_eq!(params.set_name(), Some(&"core set 2021".to_string()));
    }
}
