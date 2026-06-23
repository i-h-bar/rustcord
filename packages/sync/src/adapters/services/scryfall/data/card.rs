use crate::adapters::services::scryfall::utils::image::parse_image_id;
use crate::adapters::services::scryfall::utils::uuid::increment_uuid;
use crate::ports::storage::{
    Artist, Card, CardInfo, Combo, Illustration, Image, Legality, Price, RelatedToken, Rule, Set,
};
use serde::{Deserialize, Serialize};
use time::serde::format_description;
use time::{Date, OffsetDateTime};
use uuid::{Uuid, uuid};

format_description!(date_format, Date, "[year]-[month]-[day]");

const ANONYMOUS_ARTIST_ID: Uuid = uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
const ANONYMOUS_ARTIST: &str = "Anonymous";

#[derive(Serialize, Deserialize)]
pub struct ImageUris {
    pub png: Option<String>,
    pub art_crop: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Legalities {
    pub alchemy: String,
    pub brawl: String,
    pub commander: String,
    pub duel: String,
    pub future: String,
    pub gladiator: String,
    pub historic: String,
    pub legacy: String,
    pub modern: String,
    pub oathbreaker: String,
    pub oldschool: String,
    pub pauper: String,
    pub paupercommander: String,
    pub penny: String,
    pub pioneer: String,
    pub predh: String,
    pub premodern: String,
    pub standard: String,
    pub standardbrawl: String,
    pub timeless: String,
    pub vintage: String,
}

#[derive(Serialize, Deserialize)]
pub struct Prices {
    pub usd: Option<String>,
    pub usd_foil: Option<String>,
    pub usd_etched: Option<String>,
    pub eur: Option<String>,
    pub eur_foil: Option<String>,
    pub tix: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CardFace {
    pub name: String,
    pub oracle_id: Option<Uuid>,
    pub mana_cost: Option<String>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub colors: Option<Vec<String>>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub defense: Option<String>,
    pub flavor_text: Option<String>,
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<Uuid>>,
    pub illustration_id: Option<Uuid>,
    pub image_uris: Option<ImageUris>,
    pub produced_mana: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Component {
    Token,
    ComboPiece,
    MeldPart,
    MeldResult,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize)]
pub struct RelatedCardPart {
    pub id: Uuid,
    pub component: Component,
}

#[derive(Serialize, Deserialize)]
pub struct ScryfallCard {
    pub id: Uuid,
    pub oracle_id: Option<Uuid>,
    pub name: String,
    #[serde(with = "date_format")]
    pub released_at: Date,
    pub scryfall_uri: String,
    pub flavor_text: Option<String>,
    pub reserved: bool,
    pub rarity: String,
    pub set_id: Uuid,
    #[serde(rename = "set")]
    pub set_abbreviation: String,
    pub set_name: String,
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<Uuid>>,
    pub illustration_id: Option<Uuid>,
    pub image_uris: Option<ImageUris>,
    pub legalities: Legalities,
    pub game_changer: Option<bool>,
    pub color_identity: Vec<String>,
    pub mana_cost: Option<String>,
    pub cmc: Option<f64>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub defense: Option<String>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub colors: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub produced_mana: Option<Vec<String>>,
    pub rulings_uri: Option<String>,
    pub prices: Prices,
    pub card_faces: Option<Vec<CardFace>>,
    pub all_parts: Option<Vec<RelatedCardPart>>,
}

impl ScryfallCard {
    pub fn into_storage_records(mut self) -> Option<Vec<CardInfo>> {
        if self.card_faces.is_none() {
            return self.into_single_face_record().map(|r| vec![r]);
        }

        let faces = self.card_faces.take()?;
        if faces.len() < 2 {
            return None;
        }

        let matching_names = faces[0].name == faces[1].name;
        let mut iter = faces.into_iter();
        let front = iter.next().unwrap();
        let back = iter.next().unwrap();

        if matching_names {
            Self::produce_matching_name_records(front, back, &self)
        } else {
            Self::produce_dual_face_records(front, back, &self)
        }
    }

    fn extract_combos_for(&self, card_id: Uuid) -> Vec<Combo> {
        self.all_parts
            .iter()
            .flatten()
            .filter(|p| p.component == Component::ComboPiece)
            .map(|p| Combo {
                id: Uuid::new_v4(),
                card_id,
                combo_card_id: p.id,
            })
            .collect()
    }

    fn extract_related_tokens_for(&self, card_id: Uuid) -> Vec<RelatedToken> {
        self.all_parts
            .iter()
            .flatten()
            .filter(|p| p.component == Component::Token)
            .map(|p| RelatedToken {
                id: Uuid::new_v4(),
                card_id,
                token_id: p.id,
            })
            .collect()
    }

    fn build_legality(&self, id: Uuid) -> Legality {
        Legality {
            id,
            alchemy: self.legalities.alchemy.clone(),
            brawl: self.legalities.brawl.clone(),
            commander: self.legalities.commander.clone(),
            duel: self.legalities.duel.clone(),
            future: self.legalities.future.clone(),
            gladiator: self.legalities.gladiator.clone(),
            historic: self.legalities.historic.clone(),
            legacy: self.legalities.legacy.clone(),
            modern: self.legalities.modern.clone(),
            oathbreaker: self.legalities.oathbreaker.clone(),
            oldschool: self.legalities.oldschool.clone(),
            pauper: self.legalities.pauper.clone(),
            paupercommander: self.legalities.paupercommander.clone(),
            penny: self.legalities.penny.clone(),
            pioneer: self.legalities.pioneer.clone(),
            predh: self.legalities.predh.clone(),
            premodern: self.legalities.premodern.clone(),
            standard: self.legalities.standard.clone(),
            standardbrawl: self.legalities.standardbrawl.clone(),
            timeless: self.legalities.timeless.clone(),
            vintage: self.legalities.vintage.clone(),
            game_changer: self.game_changer.unwrap_or(false),
        }
    }

    fn build_set(&self) -> Set {
        Set {
            id: self.set_id,
            name: self.set_name.clone(),
            normalised_name: contracts::normalise::normalise(&self.set_name),
            abbreviation: self.set_abbreviation.clone(),
        }
    }

    fn build_price(&self, card_id: Uuid) -> Price {
        Price {
            id: card_id,
            usd: self.prices.usd.as_deref().and_then(|s| s.parse().ok()),
            usd_foil: self.prices.usd_foil.as_deref().and_then(|s| s.parse().ok()),
            usd_etched: self
                .prices
                .usd_etched
                .as_deref()
                .and_then(|s| s.parse().ok()),
            euro: self.prices.eur.as_deref().and_then(|s| s.parse().ok()),
            euro_foil: self.prices.eur_foil.as_deref().and_then(|s| s.parse().ok()),
            tix: self.prices.tix.as_deref().and_then(|s| s.parse().ok()),
            updated_time: OffsetDateTime::now_utc(),
        }
    }

    fn into_single_face_record(self) -> Option<CardInfo> {
        let combos = self.extract_combos_for(self.id);
        let related_tokens = self.extract_related_tokens_for(self.id);

        let image_uris = self.image_uris.as_ref()?;
        let png_url = image_uris.png.as_deref()?;
        let image_id = parse_image_id(png_url)?;
        let art_crop = image_uris.art_crop.clone();
        let png_url = png_url.to_string();
        let oracle_id = self.oracle_id?;

        // Build from &self before any partial moves
        let legality = self.build_legality(oracle_id);
        let set = self.build_set();
        let price = self.build_price(self.id);
        let artist_name = self
            .artist
            .as_deref()
            .unwrap_or(ANONYMOUS_ARTIST)
            .to_string();
        let artist_id = self
            .artist_ids
            .as_ref()
            .and_then(|ids| ids.first().copied())
            .unwrap_or(ANONYMOUS_ARTIST_ID);
        let artist_normalised = contracts::normalise::normalise(&artist_name);
        let card_normalised = contracts::normalise::normalise(&self.name);

        let image = Image {
            id: image_id,
            scryfall_url: png_url,
        };

        let illustration = self
            .illustration_id
            .zip(art_crop)
            .map(|(id, url)| Illustration {
                id,
                scryfall_url: url,
            });

        let artist = Artist {
            id: artist_id,
            name: artist_name,
            normalised_name: artist_normalised,
        };

        let card = Card {
            id: self.id,
            oracle_id,
            name: self.name.clone(),
            normalised_name: card_normalised,
            scryfall_url: self.scryfall_uri.clone(),
            flavour_text: self.flavor_text,
            release_date: self.released_at,
            reserved: self.reserved,
            rarity: self.rarity,
            artist_id,
            image_id,
            illustration_id: self.illustration_id,
            set_id: self.set_id,
            backside_id: None,
        };

        let rule = Rule {
            id: oracle_id,
            colour_identity: self.color_identity,
            mana_cost: self.mana_cost,
            cmc: self.cmc.unwrap_or(0.0),
            power: self.power,
            toughness: self.toughness,
            loyalty: self.loyalty,
            defence: self.defense,
            type_line: self.type_line,
            oracle_text: self.oracle_text,
            colours: self.colors.unwrap_or_default(),
            keywords: self.keywords.unwrap_or_default(),
            produced_mana: self.produced_mana,
            rulings_url: self.rulings_uri,
        };

        Some(CardInfo {
            card,
            artist,
            image,
            illustration,
            set,
            rule,
            legality,
            price,
            combos,
            related_tokens,
        })
    }

    fn produce_face_record(
        face: CardFace,
        card: &ScryfallCard,
        card_id: Uuid,
        oracle_id: Uuid,
        backside_id: Uuid,
    ) -> Option<CardInfo> {
        let image_uris_ref = face.image_uris.as_ref().or(card.image_uris.as_ref())?;
        let png_url = image_uris_ref.png.as_deref()?;
        let image_id = parse_image_id(png_url)?;
        let art_crop = image_uris_ref.art_crop.clone();
        let png_url = png_url.to_string();

        let image = Image {
            id: image_id,
            scryfall_url: png_url,
        };

        let illustration_id = face.illustration_id.or(card.illustration_id);
        let illustration = illustration_id.zip(art_crop).map(|(id, url)| Illustration {
            id,
            scryfall_url: url,
        });

        let artist_name = face
            .artist
            .as_deref()
            .or(card.artist.as_deref())
            .unwrap_or(ANONYMOUS_ARTIST);
        let artist = Artist {
            id: face
                .artist_ids
                .as_ref()
                .or(card.artist_ids.as_ref())
                .and_then(|ids| ids.first().copied())
                .unwrap_or(ANONYMOUS_ARTIST_ID),
            name: artist_name.to_string(),
            normalised_name: contracts::normalise::normalise(artist_name),
        };

        let card_model = Card {
            id: card_id,
            oracle_id,
            name: face.name.clone(),
            normalised_name: contracts::normalise::normalise(&face.name),
            scryfall_url: card.scryfall_uri.clone(),
            flavour_text: face.flavor_text,
            release_date: card.released_at,
            reserved: card.reserved,
            rarity: card.rarity.clone(),
            artist_id: artist.id,
            image_id,
            illustration_id,
            set_id: card.set_id,
            backside_id: Some(backside_id),
        };

        let rule = Rule {
            id: oracle_id,
            colour_identity: card.color_identity.clone(),
            mana_cost: face.mana_cost,
            cmc: card.cmc.unwrap_or(0.0),
            power: face.power,
            toughness: face.toughness,
            loyalty: face.loyalty,
            defence: face.defense,
            type_line: face.type_line,
            oracle_text: face.oracle_text,
            colours: card.colors.clone().unwrap_or_default(),
            keywords: card.keywords.clone().unwrap_or_default(),
            produced_mana: face.produced_mana,
            rulings_url: card.rulings_uri.clone(),
        };

        let legality = card.build_legality(oracle_id);
        let set = card.build_set();
        let price = card.build_price(card_id);

        Some(CardInfo {
            card: card_model,
            artist,
            image,
            illustration,
            set,
            rule,
            legality,
            price,
            combos: card.extract_combos_for(card_id),
            related_tokens: card.extract_related_tokens_for(card_id),
        })
    }

    fn produce_dual_face_records(
        front: CardFace,
        back: CardFace,
        card: &ScryfallCard,
    ) -> Option<Vec<CardInfo>> {
        let back_id = increment_uuid(card.id);
        let front_oracle_id = front.oracle_id.or(card.oracle_id)?;
        let back_oracle_id = increment_uuid(front_oracle_id);

        let front_record =
            Self::produce_face_record(front, card, card.id, front_oracle_id, back_id)?;
        let back_record = Self::produce_face_record(back, card, back_id, back_oracle_id, card.id)?;

        Some(vec![front_record, back_record])
    }

    fn produce_matching_name_records(
        front: CardFace,
        back: CardFace,
        card: &ScryfallCard,
    ) -> Option<Vec<CardInfo>> {
        let back_id = increment_uuid(card.id);
        let oracle_id = front.oracle_id.or(card.oracle_id)?;

        let front_record = Self::produce_face_record(front, card, card.id, oracle_id, back_id)?;
        let back_record = Self::produce_face_record(back, card, back_id, oracle_id, card.id)?;

        Some(vec![front_record, back_record])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    fn make_legalities() -> Legalities {
        Legalities {
            alchemy: "legal".to_string(),
            brawl: "legal".to_string(),
            commander: "legal".to_string(),
            duel: "legal".to_string(),
            future: "legal".to_string(),
            gladiator: "legal".to_string(),
            historic: "legal".to_string(),
            legacy: "legal".to_string(),
            modern: "legal".to_string(),
            oathbreaker: "legal".to_string(),
            oldschool: "not_legal".to_string(),
            pauper: "legal".to_string(),
            paupercommander: "legal".to_string(),
            penny: "legal".to_string(),
            pioneer: "legal".to_string(),
            predh: "legal".to_string(),
            premodern: "legal".to_string(),
            standard: "legal".to_string(),
            standardbrawl: "legal".to_string(),
            timeless: "legal".to_string(),
            vintage: "legal".to_string(),
        }
    }

    fn make_prices() -> Prices {
        Prices {
            usd: Some("0.35".to_string()),
            usd_foil: Some("0.40".to_string()),
            usd_etched: None,
            eur: Some("0.24".to_string()),
            eur_foil: Some("0.39".to_string()),
            tix: Some("0.03".to_string()),
        }
    }

    fn make_single_face_card() -> ScryfallCard {
        ScryfallCard {
            id: Uuid::parse_str("0000419b-0bba-4488-8f7a-6194544ce91e").unwrap(),
            oracle_id: Some(Uuid::parse_str("b34bb2dc-c1af-4d77-b0b3-a0fb342a5fc6").unwrap()),
            name: "Forest".to_string(),
            released_at: Date::from_calendar_date(2024, Month::August, 2).unwrap(),
            scryfall_uri: "https://scryfall.com/card/blb/280/forest".to_string(),
            flavor_text: None,
            reserved: false,
            rarity: "common".to_string(),
            set_id: Uuid::parse_str("a2f58272-bba6-439d-871e-7a46686ac018").unwrap(),
            set_abbreviation: "blb".to_string(),
            set_name: "Bloomburrow".to_string(),
            artist: Some("David Robert Hovey".to_string()),
            artist_ids: Some(vec![
                Uuid::parse_str("22ab27e3-6476-48f1-a9f7-9a9e86339030").unwrap(),
            ]),
            illustration_id: Some(
                Uuid::parse_str("fb2b1ca2-7440-48c2-81c8-84da0a45a626").unwrap(),
            ),
            image_uris: Some(ImageUris {
                png: Some("https://cards.scryfall.io/png/front/0/0/0000419b-0bba-4488-8f7a-6194544ce91e.png?1721427487".to_string()),
                art_crop: Some("https://cards.scryfall.io/art_crop/front/0/0/0000419b-0bba-4488-8f7a-6194544ce91e.jpg?1721427487".to_string()),
            }),
            legalities: make_legalities(),
            game_changer: Some(false),
            color_identity: vec!["G".to_string()],
            mana_cost: Some(String::new()),
            cmc: Some(0.0),
            power: None,
            toughness: None,
            loyalty: None,
            defense: None,
            type_line: Some("Basic Land \u{2014} Forest".to_string()),
            oracle_text: Some("({T}: Add {G}.)".to_string()),
            colors: Some(vec![]),
            keywords: Some(vec![]),
            produced_mana: Some(vec!["G".to_string()]),
            rulings_uri: Some("https://api.scryfall.com/cards/0000419b-0bba-4488-8f7a-6194544ce91e/rulings".to_string()),
            prices: make_prices(),
            card_faces: None,
            all_parts: None,
        }
    }

    fn make_card_face(name: &str, oracle_id: Option<&str>) -> CardFace {
        CardFace {
            name: name.to_string(),
            oracle_id: oracle_id.map(|s| Uuid::parse_str(s).unwrap()),
            mana_cost: Some("{1}{U}".to_string()),
            type_line: Some("Creature \u{2014} Human".to_string()),
            oracle_text: Some("Flying".to_string()),
            colors: Some(vec!["U".to_string()]),
            power: Some("2".to_string()),
            toughness: Some("1".to_string()),
            loyalty: None,
            defense: None,
            flavor_text: None,
            artist: Some("Artist Name".to_string()),
            artist_ids: Some(vec![
                Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
            ]),
            illustration_id: Some(
                Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
            ),
            image_uris: Some(ImageUris {
                png: Some("https://cards.scryfall.io/png/front/0/0/0000419b-0bba-4488-8f7a-6194544ce91e.png?1721427487".to_string()),
                art_crop: Some("https://cards.scryfall.io/art_crop/front/0/0/0000419b-0bba-4488-8f7a-6194544ce91e.jpg?1721427487".to_string()),
            }),
            produced_mana: None,
        }
    }

    fn make_dfc_card() -> ScryfallCard {
        let mut card = make_single_face_card();
        card.image_uris = None;
        card.illustration_id = None;
        card.card_faces = Some(vec![
            make_card_face("Front Face", Some("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")),
            make_card_face("Back Face", None),
        ]);
        card
    }

    fn make_matching_name_dfc() -> ScryfallCard {
        let mut card = make_dfc_card();
        if let Some(faces) = card.card_faces.as_mut() {
            faces[1].name = faces[0].name.clone();
        }
        card
    }

    // Single-face tests

    #[test]
    fn test_single_face_produces_one_record() {
        let records = make_single_face_card().into_storage_records().unwrap();
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn test_single_face_card_fields() {
        let records = make_single_face_card().into_storage_records().unwrap();
        let info = &records[0];
        assert_eq!(info.card.name, "Forest");
        assert_eq!(info.card.rarity, "common");
        assert!(info.card.backside_id.is_none());
        assert_eq!(
            info.card.oracle_id,
            Uuid::parse_str("b34bb2dc-c1af-4d77-b0b3-a0fb342a5fc6").unwrap()
        );
    }

    #[test]
    fn test_single_face_missing_image_returns_none() {
        let mut card = make_single_face_card();
        card.image_uris = None;
        assert!(card.into_storage_records().is_none());
    }

    #[test]
    fn test_single_face_missing_oracle_id_returns_none() {
        let mut card = make_single_face_card();
        card.oracle_id = None;
        assert!(card.into_storage_records().is_none());
    }

    #[test]
    fn test_single_face_price_parsed() {
        let records = make_single_face_card().into_storage_records().unwrap();
        let price = &records[0].price;
        assert_eq!(price.usd, Some(0.35));
        assert_eq!(price.usd_foil, Some(0.40));
        assert_eq!(price.usd_etched, None);
        assert_eq!(price.euro, Some(0.24));
    }

    #[test]
    fn test_single_face_artist() {
        let records = make_single_face_card().into_storage_records().unwrap();
        let artist = &records[0].artist;
        assert_eq!(artist.name, "David Robert Hovey");
        assert_eq!(
            artist.id,
            Uuid::parse_str("22ab27e3-6476-48f1-a9f7-9a9e86339030").unwrap()
        );
    }

    // DFC tests

    #[test]
    fn test_dfc_different_names_produces_two_records() {
        let records = make_dfc_card().into_storage_records().unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_dfc_front_keeps_original_id() {
        let card = make_dfc_card();
        let card_id = card.id;
        let records = card.into_storage_records().unwrap();
        assert_eq!(records[0].card.id, card_id);
    }

    #[test]
    fn test_dfc_back_gets_incremented_id() {
        let card = make_dfc_card();
        let expected_back_id = increment_uuid(card.id);
        let records = card.into_storage_records().unwrap();
        assert_eq!(records[1].card.id, expected_back_id);
    }

    #[test]
    fn test_dfc_different_names_have_different_oracle_ids() {
        let records = make_dfc_card().into_storage_records().unwrap();
        assert_ne!(records[0].card.oracle_id, records[1].card.oracle_id);
        assert_eq!(
            records[1].card.oracle_id,
            increment_uuid(records[0].card.oracle_id)
        );
    }

    #[test]
    fn test_dfc_front_has_back_as_backside() {
        let records = make_dfc_card().into_storage_records().unwrap();
        assert_eq!(records[0].card.backside_id, Some(records[1].card.id));
        assert_eq!(records[1].card.backside_id, Some(records[0].card.id));
    }

    #[test]
    fn test_matching_name_dfc_shares_oracle_id() {
        let records = make_matching_name_dfc().into_storage_records().unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].card.oracle_id, records[1].card.oracle_id);
    }

    #[test]
    fn test_dfc_face_name_used_not_card_name() {
        let records = make_dfc_card().into_storage_records().unwrap();
        assert_eq!(records[0].card.name, "Front Face");
        assert_eq!(records[1].card.name, "Back Face");
    }
}
