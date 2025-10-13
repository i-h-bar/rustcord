use crate::adapters::cache::Cache;
use crate::adapters::card_store::CardStore;
use crate::adapters::image_store::{ImageStore, Images};
use crate::domain::app::App;
use crate::domain::card::Card;
use crate::domain::query::QueryParams;
use crate::domain::utils::{fuzzy, REGEX_COLLECTION};
use crate::ports::clients::MessageInteraction;
use serenity::futures::future::join_all;
use tokio::time::Instant;

pub type CardAndImage = (Card, Images);

impl<IS, CS, C> App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    pub async fn parse_message(&self, msg: &str) -> Vec<Option<CardAndImage>> {
        join_all(
            REGEX_COLLECTION
                .cards
                .captures_iter(msg)
                .filter_map(|capture| Some(self.find_card(QueryParams::from(&capture)?))),
        )
        .await
    }

    async fn search_distinct_cards(&self, normalised_name: &str) -> Option<Card> {
        let potentials = self.card_store.search(normalised_name).await?;
        fuzzy::winkliest_match(&normalised_name, potentials)
    }

    pub async fn search_set_abbreviation(
        &self,
        abbreviation: &str,
        normalised_name: &str,
    ) -> Option<Card> {
        let set_name = self.set_from_abbreviation(abbreviation).await?;
        let potentials = self
            .card_store
            .search_set(&set_name, normalised_name)
            .await?;
        fuzzy::winkliest_match(&normalised_name, potentials)
    }

    async fn search_set_name(
        &self,
        normalised_set_name: &str,
        normalised_name: &str,
    ) -> Option<Card> {
        let potentials = self
            .card_store
            .search_set(normalised_set_name, normalised_name)
            .await?;
        fuzzy::winkliest_match(&normalised_name, potentials)
    }

    async fn search_artist(&self, artist: &str, normalised_name: &str) -> Option<Card> {
        let potentials = self
            .card_store
            .search_artist(artist, normalised_name)
            .await?;
        fuzzy::winkliest_match(&normalised_name, potentials)
    }

    pub async fn find_card(&self, query: QueryParams) -> Option<CardAndImage> {
        let start = Instant::now();

        let found_card = if let Some(set_code) = query.set_code() {
            self.search_set_abbreviation(set_code, query.name()).await?
        } else if let Some(set_name) = query.set_name() {
            self.search_set_name(set_name, query.name()).await?
        } else if let Some(artist) = query.artist() {
            self.search_artist(artist, query.name()).await?
        } else {
            self.search_distinct_cards(query.name()).await?
        };

        log::info!(
            "Found match for query '{}' -> '{}' in {} ms",
            query.name(),
            found_card.front_name,
            start.elapsed().as_millis()
        );

        let images = self.image_store.fetch(&found_card).await.ok()?;

        Some((found_card, images))
    }

    pub async fn fuzzy_match_set_name(&self, normalised_set_name: &str) -> Option<String> {
        let potentials = self
            .card_store
            .search_for_set_name(normalised_set_name)
            .await?;
        fuzzy::winkliest_match(&normalised_set_name, potentials)
    }

    pub async fn set_from_abbreviation(&self, abbreviation: &str) -> Option<String> {
        self.card_store
            .set_name_from_abbreviation(abbreviation)
            .await
    }

    pub async fn search<I: MessageInteraction>(&self, interaction: &I, query_params: QueryParams) {
        let card = self.find_card(query_params).await;
        if let Some((card, images)) = card {
            if let Err(why) = interaction.send_card(card, images).await {
                log::warn!("Error sending card from search command: {why}");
            };
        } else if let Err(why) = interaction
            .reply(String::from("Could not find card :("))
            .await
        {
            log::warn!("Error the failed to find card message from search command: {why}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::cache::MockCache;
    use crate::adapters::card_store::MockCardStore;
    use crate::adapters::image_store::{Images, MockImageStore};
    use crate::ports::clients::MockMessageInteraction;
    use mockall::predicate::eq;
    use uuid::uuid;

    #[tokio::test]
    async fn test_search() {
        let query = QueryParams::from_test(String::from("gitrog monster"), None, None, None);
        let front_image_id = uuid!("40489e28-878d-44a2-847f-07beef1aa0f8");
        let card = Card { front_name: "The Gitrog Monster".to_string(), front_normalised_name: "the gitrog monster".to_string(), front_scryfall_url: "https://scryfall.com/card/eoc/117/the-gitrog-monster?utm_source=api".to_string(), front_image_id, front_illustration_id: Some(uuid!("ccf210fd-8ef1-4250-ae86-66ede33614d5")), front_mana_cost: "{3}{B}{G}".to_string(), front_colour_identity: vec!["B".to_string(), "G".to_string()], front_power: Some("6".to_string()), front_toughness: Some("6".to_string()), front_loyalty: None, front_defence: None, front_type_line: "Legendary Creature — Frog Horror".to_string(), front_oracle_text: "Deathtouch\nAt the beginning of your upkeep, sacrifice The Gitrog Monster unless you sacrifice a land.\nYou may play an additional land on each of your turns.\nWhenever one or more land cards are put into your graveyard from anywhere, draw a card.".to_string(), back_name: None, back_scryfall_url: None, back_image_id: None, back_illustration_id: None, back_mana_cost: None, back_colour_identity: None, back_power: None, back_toughness: None, back_loyalty: None, back_defence: None, back_type_line: None, back_oracle_text: None, artist: "Jason Kang".to_string(), set_name: "Edge of Eternities Commander".to_string() };
        let images = Images {
            front: vec![1, 2, 3, 4],
            back: None,
        };

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch()
            .times(1)
            .with(eq(card.clone()))
            .return_const(Ok(images.clone()));

        let mut card_store = MockCardStore::new();
        card_store
            .expect_search()
            .times(1)
            .with(eq(query.clone().name().clone()))
            .return_const(Some(vec![card.clone()]));

        let cache = MockCache::new();
        let mut interaction = MockMessageInteraction::new();
        interaction
            .expect_send_card()
            .times(1)
            .with(eq(card.clone()), eq(images.clone()))
            .return_const(Ok(()));

        let app = App::new(image_store, card_store, cache);

        app.search(&interaction, query.clone()).await;
    }

    #[tokio::test]
    async fn test_search_card_not_found() {
        let query = QueryParams::from_test(String::from("nonexistent card"), None, None, None);

        let image_store = MockImageStore::new();

        let mut card_store = MockCardStore::new();
        card_store
            .expect_search()
            .times(1)
            .with(eq(query.name().clone()))
            .return_const(None);

        let cache = MockCache::new();
        let mut interaction = MockMessageInteraction::new();
        interaction
            .expect_reply()
            .times(1)
            .with(eq(String::from("Could not find card :(")))
            .return_const(Ok(()));

        let app = App::new(image_store, card_store, cache);

        app.search(&interaction, query.clone()).await;
    }

    #[tokio::test]
    async fn test_search_with_set_code() {
        let query = QueryParams::from_test(
            String::from("lightning bolt"),
            None,
            None,
            Some(String::from("LEA")),
        );
        let card = Card {
            front_name: "Lightning Bolt".to_string(),
            front_normalised_name: "lightning bolt".to_string(),
            front_scryfall_url: "https://scryfall.com/card/test".to_string(),
            front_image_id: uuid!("12345678-1234-1234-1234-123456789012"),
            front_illustration_id: Some(uuid!("12345678-1234-1234-1234-123456789013")),
            front_mana_cost: "{R}".to_string(),
            front_colour_identity: vec!["R".to_string()],
            front_power: None,
            front_toughness: None,
            front_loyalty: None,
            front_defence: None,
            front_type_line: "Instant".to_string(),
            front_oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
            back_name: None,
            back_scryfall_url: None,
            back_image_id: None,
            back_illustration_id: None,
            back_mana_cost: None,
            back_colour_identity: None,
            back_power: None,
            back_toughness: None,
            back_loyalty: None,
            back_defence: None,
            back_type_line: None,
            back_oracle_text: None,
            artist: "Christopher Rush".to_string(),
            set_name: "Limited Edition Alpha".to_string(),
        };
        let images = Images {
            front: vec![1, 2, 3, 4],
            back: None,
        };

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch()
            .times(1)
            .return_const(Ok(images.clone()));

        let mut card_store = MockCardStore::new();
        card_store
            .expect_set_name_from_abbreviation()
            .times(1)
            .with(eq("LEA"))
            .return_const(Some("Limited Edition Alpha".to_string()));
        card_store
            .expect_search_set()
            .times(1)
            .with(eq("Limited Edition Alpha"), eq(query.name().clone()))
            .return_const(Some(vec![card.clone()]));

        let cache = MockCache::new();
        let mut interaction = MockMessageInteraction::new();
        interaction
            .expect_send_card()
            .times(1)
            .return_const(Ok(()));

        let app = App::new(image_store, card_store, cache);

        app.search(&interaction, query.clone()).await;
    }

    #[tokio::test]
    async fn test_search_with_artist() {
        let query = QueryParams::from_test(
            String::from("lightning bolt"),
            Some(String::from("Christopher Rush")),
            None,
            None,
        );
        let card = Card {
            front_name: "Lightning Bolt".to_string(),
            front_normalised_name: "lightning bolt".to_string(),
            front_scryfall_url: "https://scryfall.com/card/test".to_string(),
            front_image_id: uuid!("12345678-1234-1234-1234-123456789012"),
            front_illustration_id: Some(uuid!("12345678-1234-1234-1234-123456789013")),
            front_mana_cost: "{R}".to_string(),
            front_colour_identity: vec!["R".to_string()],
            front_power: None,
            front_toughness: None,
            front_loyalty: None,
            front_defence: None,
            front_type_line: "Instant".to_string(),
            front_oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
            back_name: None,
            back_scryfall_url: None,
            back_image_id: None,
            back_illustration_id: None,
            back_mana_cost: None,
            back_colour_identity: None,
            back_power: None,
            back_toughness: None,
            back_loyalty: None,
            back_defence: None,
            back_type_line: None,
            back_oracle_text: None,
            artist: "Christopher Rush".to_string(),
            set_name: "Limited Edition Alpha".to_string(),
        };
        let images = Images {
            front: vec![1, 2, 3, 4],
            back: None,
        };

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch()
            .times(1)
            .return_const(Ok(images.clone()));

        let mut card_store = MockCardStore::new();
        card_store
            .expect_search_artist()
            .times(1)
            .with(eq("Christopher Rush"), eq(query.name().clone()))
            .return_const(Some(vec![card.clone()]));

        let cache = MockCache::new();
        let mut interaction = MockMessageInteraction::new();
        interaction
            .expect_send_card()
            .times(1)
            .return_const(Ok(()));

        let app = App::new(image_store, card_store, cache);

        app.search(&interaction, query.clone()).await;
    }

    #[tokio::test]
    async fn test_parse_message_single_card() {
        let card = Card {
            front_name: "Lightning Bolt".to_string(),
            front_normalised_name: "lightning bolt".to_string(),
            front_scryfall_url: "https://scryfall.com/card/test".to_string(),
            front_image_id: uuid!("12345678-1234-1234-1234-123456789012"),
            front_illustration_id: Some(uuid!("12345678-1234-1234-1234-123456789013")),
            front_mana_cost: "{R}".to_string(),
            front_colour_identity: vec!["R".to_string()],
            front_power: None,
            front_toughness: None,
            front_loyalty: None,
            front_defence: None,
            front_type_line: "Instant".to_string(),
            front_oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
            back_name: None,
            back_scryfall_url: None,
            back_image_id: None,
            back_illustration_id: None,
            back_mana_cost: None,
            back_colour_identity: None,
            back_power: None,
            back_toughness: None,
            back_loyalty: None,
            back_defence: None,
            back_type_line: None,
            back_oracle_text: None,
            artist: "Christopher Rush".to_string(),
            set_name: "Limited Edition Alpha".to_string(),
        };
        let images = Images {
            front: vec![1, 2, 3, 4],
            back: None,
        };

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch()
            .times(1)
            .return_const(Ok(images.clone()));

        let mut card_store = MockCardStore::new();
        card_store
            .expect_search()
            .times(1)
            .with(eq("lightning bolt"))
            .return_const(Some(vec![card.clone()]));

        let cache = MockCache::new();
        let app = App::new(image_store, card_store, cache);

        let results = app.parse_message("Check out [[Lightning Bolt]]!").await;

        assert_eq!(results.len(), 1);
        assert!(results[0].is_some());
        if let Some((found_card, _)) = &results[0] {
            assert_eq!(found_card.front_name, "Lightning Bolt");
        }
    }

    #[tokio::test]
    async fn test_parse_message_multiple_cards() {
        let bolt_card = Card {
            front_name: "Lightning Bolt".to_string(),
            front_normalised_name: "lightning bolt".to_string(),
            front_scryfall_url: "https://scryfall.com/card/test1".to_string(),
            front_image_id: uuid!("12345678-1234-1234-1234-123456789012"),
            front_illustration_id: Some(uuid!("12345678-1234-1234-1234-123456789013")),
            front_mana_cost: "{R}".to_string(),
            front_colour_identity: vec!["R".to_string()],
            front_power: None,
            front_toughness: None,
            front_loyalty: None,
            front_defence: None,
            front_type_line: "Instant".to_string(),
            front_oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
            back_name: None,
            back_scryfall_url: None,
            back_image_id: None,
            back_illustration_id: None,
            back_mana_cost: None,
            back_colour_identity: None,
            back_power: None,
            back_toughness: None,
            back_loyalty: None,
            back_defence: None,
            back_type_line: None,
            back_oracle_text: None,
            artist: "Christopher Rush".to_string(),
            set_name: "Limited Edition Alpha".to_string(),
        };

        let giant_card = Card {
            front_name: "Giant Growth".to_string(),
            front_normalised_name: "giant growth".to_string(),
            front_scryfall_url: "https://scryfall.com/card/test2".to_string(),
            front_image_id: uuid!("22345678-1234-1234-1234-123456789012"),
            front_illustration_id: Some(uuid!("22345678-1234-1234-1234-123456789013")),
            front_mana_cost: "{G}".to_string(),
            front_colour_identity: vec!["G".to_string()],
            front_power: None,
            front_toughness: None,
            front_loyalty: None,
            front_defence: None,
            front_type_line: "Instant".to_string(),
            front_oracle_text: "Target creature gets +3/+3 until end of turn.".to_string(),
            back_name: None,
            back_scryfall_url: None,
            back_image_id: None,
            back_illustration_id: None,
            back_mana_cost: None,
            back_colour_identity: None,
            back_power: None,
            back_toughness: None,
            back_loyalty: None,
            back_defence: None,
            back_type_line: None,
            back_oracle_text: None,
            artist: "Mark Poole".to_string(),
            set_name: "Limited Edition Alpha".to_string(),
        };

        let images = Images {
            front: vec![1, 2, 3, 4],
            back: None,
        };

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch()
            .times(2)
            .returning(move |_| Ok(images.clone()));

        let mut card_store = MockCardStore::new();
        card_store
            .expect_search()
            .times(1)
            .with(eq("lightning bolt"))
            .return_const(Some(vec![bolt_card.clone()]));
        card_store
            .expect_search()
            .times(1)
            .with(eq("giant growth"))
            .return_const(Some(vec![giant_card.clone()]));

        let cache = MockCache::new();
        let app = App::new(image_store, card_store, cache);

        let results = app
            .parse_message("I love [[Lightning Bolt]] and [[Giant Growth]]!")
            .await;

        assert_eq!(results.len(), 2);
        assert!(results[0].is_some());
        assert!(results[1].is_some());
    }

    #[tokio::test]
    async fn test_parse_message_no_cards() {
        let image_store = MockImageStore::new();
        let card_store = MockCardStore::new();
        let cache = MockCache::new();
        let app = App::new(image_store, card_store, cache);

        let results = app
            .parse_message("This message has no card references")
            .await;

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_find_card_with_set_name() {
        let query = QueryParams::from_test(
            String::from("lightning bolt"),
            None,
            Some(String::from("limited edition alpha")),
            None,
        );
        let card = Card {
            front_name: "Lightning Bolt".to_string(),
            front_normalised_name: "lightning bolt".to_string(),
            front_scryfall_url: "https://scryfall.com/card/test".to_string(),
            front_image_id: uuid!("12345678-1234-1234-1234-123456789012"),
            front_illustration_id: Some(uuid!("12345678-1234-1234-1234-123456789013")),
            front_mana_cost: "{R}".to_string(),
            front_colour_identity: vec!["R".to_string()],
            front_power: None,
            front_toughness: None,
            front_loyalty: None,
            front_defence: None,
            front_type_line: "Instant".to_string(),
            front_oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
            back_name: None,
            back_scryfall_url: None,
            back_image_id: None,
            back_illustration_id: None,
            back_mana_cost: None,
            back_colour_identity: None,
            back_power: None,
            back_toughness: None,
            back_loyalty: None,
            back_defence: None,
            back_type_line: None,
            back_oracle_text: None,
            artist: "Christopher Rush".to_string(),
            set_name: "Limited Edition Alpha".to_string(),
        };
        let images = Images {
            front: vec![1, 2, 3, 4],
            back: None,
        };

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch()
            .times(1)
            .return_const(Ok(images.clone()));

        let mut card_store = MockCardStore::new();
        card_store
            .expect_search_set()
            .times(1)
            .with(
                eq("limited edition alpha"),
                eq("lightning bolt"),
            )
            .return_const(Some(vec![card.clone()]));

        let cache = MockCache::new();
        let app = App::new(image_store, card_store, cache);

        let result = app.find_card(query).await;

        assert!(result.is_some());
        if let Some((found_card, _)) = result {
            assert_eq!(found_card.front_name, "Lightning Bolt");
        }
    }

    #[tokio::test]
    async fn test_fuzzy_match_set_name() {
        let mut card_store = MockCardStore::new();
        card_store
            .expect_search_for_set_name()
            .times(1)
            .with(eq("limited edition alpha"))
            .return_const(Some(vec![
                "Limited Edition Alpha".to_string(),
                "Limited Edition Beta".to_string(),
            ]));

        let image_store = MockImageStore::new();
        let cache = MockCache::new();
        let app = App::new(image_store, card_store, cache);

        let result = app.fuzzy_match_set_name("limited edition alpha").await;

        assert_eq!(result, Some("Limited Edition Alpha".to_string()));
    }
}
