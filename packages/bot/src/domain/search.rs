use crate::domain::app::App;
use contracts::search_result::SearchResultDto;
use contracts::card::Card;
use crate::domain::query::QueryParams;
use crate::domain::utils::REGEX_COLLECTION;
use crate::ports::drivers::client::MessageInteraction;
use crate::ports::services::cache::Cache;
use crate::ports::services::card_store::CardStore;
use crate::ports::services::image_store::ImageStore;
use serenity::futures::future::join_all;
use tokio::time::Instant;
use uuid::Uuid;
use fuzzy;

impl<IS, CS, C> App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    pub async fn parse_message(&self, msg: &str) -> Vec<Option<SearchResultDto>> {
        join_all(
            REGEX_COLLECTION
                .cards
                .captures_iter(msg)
                .filter_map(|capture| Some(self.find_card(QueryParams::from(&capture)?))),
        )
        .await
    }

    async fn search_distinct_cards(&self, normalised_name: &str) -> Option<Vec<Card>> {
        let potentials = self.card_store.search(normalised_name).await?;
        Some(fuzzy::winkliest_sort(&normalised_name, potentials))
    }

    pub async fn search_set_abbreviation(
        &self,
        abbreviation: &str,
        normalised_name: &str,
    ) -> Option<Vec<Card>> {
        let set_name = self.set_from_abbreviation(abbreviation).await?;
        let potentials = self
            .card_store
            .search_set(&set_name, normalised_name)
            .await?;
        Some(fuzzy::winkliest_sort(&normalised_name, potentials))
    }

    async fn search_set_name(
        &self,
        normalised_set_name: &str,
        normalised_name: &str,
    ) -> Option<Vec<Card>> {
        let potentials = self
            .card_store
            .search_set(normalised_set_name, normalised_name)
            .await?;
        Some(fuzzy::winkliest_sort(&normalised_name, potentials))
    }

    async fn search_artist(&self, artist: &str, normalised_name: &str) -> Option<Vec<Card>> {
        let potentials = self
            .card_store
            .search_artist(artist, normalised_name)
            .await?;
        Some(fuzzy::winkliest_sort(&normalised_name, potentials))
    }

    pub async fn find_card(&self, query: QueryParams) -> Option<SearchResultDto> {
        let start = Instant::now();

        let mut found_cards = if let Some(set_code) = query.set_code() {
            self.search_set_abbreviation(set_code, query.name()).await?
        } else if let Some(set_name) = query.set_name() {
            self.search_set_name(set_name, query.name()).await?
        } else if let Some(artist) = query.artist() {
            self.search_artist(artist, query.name()).await?
        } else {
            self.search_distinct_cards(query.name()).await?
        };

        let found_card = found_cards.drain(0..1).next()?;

        log::info!(
            "Found match for query '{}' -> '{}' in {} ms",
            query.name(),
            found_card.name(),
            start.elapsed().as_millis()
        );

        let (sets, images) = tokio::join!(
            self.card_store.all_prints(found_card.oracle_id()),
            self.image_store.fetch(&found_card),
        );

        Some(
            SearchResultDto::new(found_card, images.ok()?)
                .add_printings(sets)
                .add_similar_cards(found_cards),
        )
    }

    pub async fn fetch_from_id(&self, card_id: &Uuid) -> Option<SearchResultDto> {
        let start = Instant::now();
        let card = self.card_store.fetch_card_by_id(card_id).await?;
        let similar_cards = self.card_store.similar_cards(card.normalised_name()).await?;
        let (sets, images) = tokio::join!(
            self.card_store.all_prints(card.oracle_id()),
            self.image_store.fetch(&card),
        );
        log::info!("Fetch new print in {}ms", start.elapsed().as_millis());
        Some(SearchResultDto::new(card, images.ok()?).add_printings(sets).add_similar_cards(similar_cards))
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
        let result = self.find_card(query_params).await;
        if let Some(result) = result {
            if let Err(why) = interaction.send_card(result).await {
                log::warn!("Error sending card from search command: {why}");
            };
        } else if let Err(why) = interaction
            .reply(String::from("Could not find card :("))
            .await
        {
            log::warn!("Error the failed to find card message from search command: {why}");
        }
    }

    pub async fn select_print<I: MessageInteraction>(&self, interaction: &I, card_id: Uuid) {
        match self.fetch_from_id(&card_id).await {
            Some(result) => {
                if let Err(why) = interaction.send_card(result).await {
                    log::warn!("Error updating card for print selection: {why}");
                }
            }
            None => {
                if let Err(why) = interaction
                    .reply(String::from("Could not find that print :("))
                    .await
                {
                    log::warn!("Error sending print not found: {why}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::drivers::client::MockMessageInteraction;
    use crate::ports::services::cache::MockCache;
    use crate::ports::services::card_store::MockCardStore;
    use contracts::image::Image;
    use crate::ports::services::image_store::MockImageStore;
    use mockall::predicate::eq;
    use uuid::uuid;

    #[tokio::test]
    async fn test_search() {
        let query = QueryParams::from_test(String::from("gitrog monster"), None, None, None);
        let front_image_id = uuid!("40489e28-878d-44a2-847f-07beef1aa0f8");
        let card = Card::new(
            front_image_id,
            "The Gitrog Monster".to_string(),
            "the gitrog monster".to_string(),
            front_image_id,
            "https://scryfall.com/card/eoc/117/the-gitrog-monster?utm_source=api".to_string(),
            front_image_id,
            Some(uuid!("ccf210fd-8ef1-4250-ae86-66ede33614d5")),
            "{3}{B}{G}".to_string(),
            vec!["B".to_string(), "G".to_string()],
            Some("6".to_string()),
            Some("6".to_string()),
            None,
            None,
            "Legendary Creature — Frog Horror".to_string(),
            "Deathtouch\nAt the beginning of your upkeep, sacrifice The Gitrog Monster unless you sacrifice a land.\nYou may play an additional land on each of your turns.\nWhenever one or more land cards are put into your graveyard from anywhere, draw a card.".to_string(),
            None,
            "Jason Kang".to_string(),
            "Edge of Eternities Commander".to_string(),
        );
        let images = Image::new(vec![1, 2, 3, 4]);

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
        card_store.expect_all_prints().returning(|_| None);

        let cache = MockCache::new();
        let mut interaction = MockMessageInteraction::new();
        interaction.expect_send_card().times(1).return_const(Ok(()));

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
        let card = make_test_card(uuid!("12345678-1234-1234-1234-123456789012"), "Lightning Bolt", "lightning bolt", "Limited Edition Alpha");
        let images = Image::new(vec![1, 2, 3, 4]);

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
        card_store.expect_all_prints().returning(|_| None);

        let cache = MockCache::new();
        let mut interaction = MockMessageInteraction::new();
        interaction.expect_send_card().times(1).return_const(Ok(()));

        let app = App::new(image_store, card_store, cache);

        app.search(&interaction, query.clone()).await;
    }

    fn make_test_card(id: uuid::Uuid, name: &str, normalised_name: &str, set_name: &str) -> Card {
        Card::new(
            id,
            name.to_string(),
            normalised_name.to_string(),
            id,
            "https://scryfall.com/card/test".to_string(),
            id,
            Some(uuid::Uuid::from_u128(id.as_u128() + 1)),
            "{R}".to_string(),
            vec!["R".to_string()],
            None,
            None,
            None,
            None,
            "Instant".to_string(),
            "Lightning Bolt deals 3 damage to any target.".to_string(),
            None,
            "Christopher Rush".to_string(),
            set_name.to_string(),
        )
    }

    #[tokio::test]
    async fn test_search_with_artist() {
        let query = QueryParams::from_test(
            String::from("lightning bolt"),
            Some(String::from("Christopher Rush")),
            None,
            None,
        );
        let card = make_test_card(
            uuid!("12345678-1234-1234-1234-123456789012"),
            "Lightning Bolt",
            "lightning bolt",
            "Limited Edition Alpha",
        );
        let images = Image::new(vec![1, 2, 3, 4]);

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
        card_store.expect_all_prints().returning(|_| None);

        let cache = MockCache::new();
        let mut interaction = MockMessageInteraction::new();
        interaction.expect_send_card().times(1).return_const(Ok(()));

        let app = App::new(image_store, card_store, cache);

        app.search(&interaction, query.clone()).await;
    }

    #[tokio::test]
    async fn test_parse_message_single_card() {
        let card = make_test_card(
            uuid!("12345678-1234-1234-1234-123456789012"),
            "Lightning Bolt",
            "lightning bolt",
            "Limited Edition Alpha",
        );
        let images = Image::new(vec![1, 2, 3, 4]);

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
        card_store.expect_all_prints().returning(|_| None);

        let cache = MockCache::new();
        let app = App::new(image_store, card_store, cache);

        let results = app.parse_message("Check out [[Lightning Bolt]]!").await;

        assert_eq!(results.len(), 1);
        assert!(results[0].is_some());
        if let Some(result) = &results[0] {
            assert_eq!(result.card().name(), "Lightning Bolt");
        }
    }

    #[tokio::test]
    async fn test_parse_message_multiple_cards() {
        let bolt_card = make_test_card(
            uuid!("12345678-1234-1234-1234-123456789012"),
            "Lightning Bolt",
            "lightning bolt",
            "Limited Edition Alpha",
        );
        let giant_card = make_test_card(
            uuid!("22345678-1234-1234-1234-123456789012"),
            "Giant Growth",
            "giant growth",
            "Limited Edition Alpha",
        );

        let images = Image::new(vec![1, 2, 3, 4]);

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
        card_store.expect_all_prints().times(2).returning(|_| None);

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
        let card = make_test_card(
            uuid!("12345678-1234-1234-1234-123456789012"),
            "Lightning Bolt",
            "lightning bolt",
            "Limited Edition Alpha",
        );
        let images = Image::new(vec![1, 2, 3, 4]);

        let mut image_store = MockImageStore::new();
        image_store
            .expect_fetch()
            .times(1)
            .return_const(Ok(images.clone()));

        let mut card_store = MockCardStore::new();
        card_store
            .expect_search_set()
            .times(1)
            .with(eq("limited edition alpha"), eq("lightning bolt"))
            .return_const(Some(vec![card.clone()]));
        card_store.expect_all_prints().returning(|_| None);

        let cache = MockCache::new();
        let app = App::new(image_store, card_store, cache);

        let result = app.find_card(query).await;

        assert!(result.is_some());
        if let Some(result) = result {
            assert_eq!(result.card().name(), "Lightning Bolt");
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
