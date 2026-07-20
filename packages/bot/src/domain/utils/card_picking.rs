use contracts::card::Card;
use fuzzy::ToBytes;

/// Local wrapper letting `Card` plug into `fuzzy::winkliest_sort`'s
/// `ToBytes` bound without giving `contracts` a dependency on `fuzzy` (or
/// vice versa) just for this one trait impl.
struct CardByName(Card);

impl ToBytes for CardByName {
    fn to_bytes(&self) -> &[u8] {
        self.0.normalised_name().as_bytes()
    }
}

#[must_use]
pub fn fuzzy_sort(needle: &str, haystack: Vec<Card>) -> Vec<Card> {
    fuzzy::winkliest_sort(&needle, haystack.into_iter().map(CardByName))
        .into_iter()
        .map(|CardByName(card)| card)
        .collect()
}

#[must_use]
pub fn extract_match(haystack: Vec<Card>, needle: &str) -> Option<(Card, Vec<Card>)> {
    let mut found_cards_sorted = fuzzy_sort(needle, haystack);

    let found_index: usize = {
        let top_card = found_cards_sorted.first()?;
        if top_card.back_id().is_none() {
            0
        } else {
            let top_name = top_card.name();
            let mut result = None;
            for (i, card) in found_cards_sorted.iter().skip(1).enumerate() {
                if card.name() != top_name {
                    break;
                }
                if card.back_id().is_none() {
                    result = Some(i + 1);
                    break;
                }
            }
            result.unwrap_or(0)
        }
    };

    let found_card = found_cards_sorted.remove(found_index);
    Some((found_card, found_cards_sorted))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{uuid, Uuid};

    const BACK_UUID: Uuid = uuid!("ffffffff-ffff-ffff-ffff-ffffffffffff");

    fn make_card(id: Uuid, name: &str, back_id: Option<Uuid>) -> Card {
        Card::new(
            id,
            name.to_string(),
            name.to_lowercase(),
            id,
            String::new(),
            id,
            None, // illustration_id
            String::new(),
            vec![],
            None,
            None,
            None,
            None,
            String::new(),
            String::new(),
            back_id, // back_id
            String::new(),
            String::new(),
            String::new(),
            time::Date::from_calendar_date(1993, time::Month::August, 5).unwrap(),
        )
    }

    #[test]
    fn fuzzy_sort_orders_by_descending_similarity() {
        // Pinning test: fuzzy_sort must order candidates by descending
        // Jaro-Winkler similarity to the needle, regardless of internal
        // implementation (hand-rolled scalar sort vs. fuzzy::winkliest_sort).
        let bolt = make_card(
            uuid!("00000000-0000-0000-0000-000000000001"),
            "Lightning Bolt",
            None,
        );
        let strike = make_card(
            uuid!("00000000-0000-0000-0000-000000000002"),
            "Lightning Strike",
            None,
        );
        let chain = make_card(
            uuid!("00000000-0000-0000-0000-000000000003"),
            "Chain Lightning",
            None,
        );

        let sorted = fuzzy_sort("lightning bolt", vec![chain, strike, bolt]);

        let names: Vec<&str> = sorted.iter().map(Card::name).collect();
        assert_eq!(
            names,
            vec!["Lightning Bolt", "Lightning Strike", "Chain Lightning"]
        );
    }

    #[test]
    fn returns_none_for_empty_haystack() {
        assert!(extract_match(vec![], "lightning bolt").is_none());
    }

    #[test]
    fn single_card_no_backside_is_returned() {
        let card = make_card(
            uuid!("00000000-0000-0000-0000-000000000001"),
            "Lightning Bolt",
            None,
        );
        let (found, discarded) = extract_match(vec![card], "lightning bolt").unwrap();
        assert_eq!(found.name(), "Lightning Bolt");
        assert!(found.back_id().is_none());
        assert!(discarded.is_empty());
    }

    #[test]
    fn single_card_with_backside_is_returned() {
        let card = make_card(
            uuid!("00000000-0000-0000-0000-000000000001"),
            "Lightning Bolt",
            Some(BACK_UUID),
        );
        let (found, discarded) = extract_match(vec![card], "lightning bolt").unwrap();
        assert_eq!(found.name(), "Lightning Bolt");
        assert!(found.back_id().is_some());
        assert!(discarded.is_empty());
    }

    #[test]
    fn first_card_no_backside_returned_without_entering_loop() {
        let bolt = make_card(
            uuid!("00000000-0000-0000-0000-000000000001"),
            "Lightning Bolt",
            None,
        );
        let rift = make_card(
            uuid!("00000000-0000-0000-0000-000000000002"),
            "Lightning Rift",
            None,
        );
        let (found, discarded) = extract_match(vec![bolt, rift], "lightning bolt").unwrap();
        assert_eq!(found.name(), "Lightning Bolt");
        assert!(found.back_id().is_none());
        assert_eq!(discarded.len(), 1);
    }

    #[test]
    fn prefers_same_named_card_without_backside() {
        let with_back = make_card(
            uuid!("00000000-0000-0000-0000-000000000001"),
            "Lightning Bolt",
            Some(BACK_UUID),
        );
        let without_back = make_card(
            uuid!("00000000-0000-0000-0000-000000000002"),
            "Lightning Bolt",
            None,
        );
        let (found, discarded) =
            extract_match(vec![with_back, without_back], "lightning bolt").unwrap();
        assert!(found.back_id().is_none());
        assert_eq!(discarded.len(), 1);
        assert!(discarded[0].back_id().is_some());
    }

    #[test]
    fn returns_first_when_all_same_named_cards_have_backsides() {
        let back1 = make_card(
            uuid!("00000000-0000-0000-0000-000000000001"),
            "Lightning Bolt",
            Some(BACK_UUID),
        );
        let back2 = make_card(
            uuid!("00000000-0000-0000-0000-000000000002"),
            "Lightning Bolt",
            Some(BACK_UUID),
        );
        let (found, discarded) = extract_match(vec![back1, back2], "lightning bolt").unwrap();
        assert!(found.back_id().is_some());
        assert_eq!(discarded.len(), 1);
    }

    #[test]
    fn backside_preference_does_not_cross_name_boundary() {
        let bolt = make_card(
            uuid!("00000000-0000-0000-0000-000000000001"),
            "Lightning Bolt",
            Some(BACK_UUID),
        );
        let recall = make_card(
            uuid!("00000000-0000-0000-0000-000000000002"),
            "Ancestral Recall",
            None,
        );
        let (found, discarded) = extract_match(vec![bolt, recall], "lightning bolt").unwrap();
        assert_eq!(found.name(), "Lightning Bolt");
        assert!(found.back_id().is_some());
        assert_eq!(discarded[0].name(), "Ancestral Recall");
    }
}
