# ScryfallCard → Storage Records Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert `ScryfallCard` (raw Scryfall API response) into DB-ready `CardInfo` storage records, including correct handling of single-face and dual-faced cards (DFCs), and write them to PostgreSQL via the `Storage` port.

**Architecture:** Mirrors the Python `db` package's `CardInfo.parse_card` logic. Storage structs live in `ports/storage.rs` (the storage port contract). UUID and image-ID utilities live in `adapters/services/scryfall/utils/`. Conversion logic lives as `impl ScryfallCard` in `adapters/services/scryfall/data/card.rs`. The PostgreSQL adapter implements the new `upsert_cards` method on the `Storage` trait.

**Tech Stack:** Rust · sqlx 0.8 (postgres, uuid, time features) · regex 1.x · time 0.3 · uuid 1.18 · contracts crate (normalise)

---

## File Map

| Action | Path | Responsibility |
|--------|------|----------------|
| Modify | `packages/spoilers_upload/src/ports/storage.rs` | Add all storage structs + `upsert_cards` to trait |
| Modify | `packages/spoilers_upload/src/ports/source.rs` | Return `Vec<CardInfo>` instead of `Vec<Card>` |
| **Create** | `packages/spoilers_upload/src/adapters/services/scryfall/utils/mod.rs` | Declare uuid + image submodules |
| **Create** | `packages/spoilers_upload/src/adapters/services/scryfall/utils/uuid.rs` | `increment_uuid` |
| **Create** | `packages/spoilers_upload/src/adapters/services/scryfall/utils/image.rs` | `parse_image_id` |
| Modify | `packages/spoilers_upload/src/adapters/services/scryfall/mod.rs` | Declare `pub mod utils;`, wire conversion |
| Modify | `packages/spoilers_upload/src/adapters/services/scryfall/data/card.rs` | Add `into_storage_records` + helpers |
| Modify | `packages/spoilers_upload/src/adapters/services/psql/mod.rs` | Implement `upsert_cards` |
| Modify | `packages/spoilers_upload/src/main.rs` | Call `upsert_cards` |
| Modify | `packages/spoilers_upload/Cargo.toml` | Add `regex = "1"` dependency |

---

## Background: The Three Conversion Paths

```
ScryfallCard.card_faces == None          → single CardInfo (backside_id = None)
card_faces[0].name == card_faces[1].name → two CardInfo sharing the same oracle_id
card_faces[0].name != card_faces[1].name → two CardInfo, back gets increment_uuid(front_oracle_id)
```

For DFCs the back card's `id = increment_uuid(front_card.id)`. Image IDs are parsed from the
Scryfall CDN URL — back-face URLs also get `increment_uuid` applied to the extracted UUID.

Anonymous artist fallback UUID: `aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa` (matches Python).

DB insert order (FK constraint order): artist → image → illustration → set → rule → legality → card → price.

---

## Task 1: Storage structs and updated port contracts

**Files:**
- Modify: `packages/spoilers_upload/src/ports/storage.rs`
- Modify: `packages/spoilers_upload/src/ports/source.rs`

- [ ] **Step 1: Replace `ports/storage.rs` with full struct definitions**

```rust
use async_trait::async_trait;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

#[async_trait]
pub trait Storage {
    async fn get_set_volumes(&self, sets: Vec<Set>) -> Vec<(Set, u32)>;
    async fn upsert_cards(&self, cards: Vec<CardInfo>);
}

pub struct Set {
    pub id: Uuid,
    pub name: String,
    pub normalised_name: String,
    pub abbreviation: String,
}

pub struct Artist {
    pub id: Uuid,
    pub name: String,
    pub normalised_name: String,
}

pub struct Image {
    pub id: Uuid,
    pub scryfall_url: String,
}

pub struct Illustration {
    pub id: Uuid,
    pub scryfall_url: String,
}

pub struct Legality {
    pub id: Uuid,
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
    pub game_changer: bool,
}

pub struct Rule {
    pub id: Uuid,
    pub colour_identity: Vec<String>,
    pub mana_cost: Option<String>,
    pub cmc: f64,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub defence: Option<String>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub colours: Vec<String>,
    pub keywords: Vec<String>,
    pub produced_mana: Option<Vec<String>>,
    pub rulings_url: Option<String>,
}

pub struct Card {
    pub id: Uuid,
    pub oracle_id: Uuid,
    pub name: String,
    pub normalised_name: String,
    pub scryfall_url: String,
    pub flavour_text: Option<String>,
    pub release_date: Date,
    pub reserved: bool,
    pub rarity: String,
    pub artist_id: Uuid,
    pub image_id: Uuid,
    pub illustration_id: Option<Uuid>,
    pub set_id: Uuid,
    pub backside_id: Option<Uuid>,
}

pub struct Price {
    pub id: Uuid,
    pub usd: Option<f64>,
    pub usd_foil: Option<f64>,
    pub usd_etched: Option<f64>,
    pub euro: Option<f64>,
    pub euro_foil: Option<f64>,
    pub tix: Option<f64>,
    pub updated_time: OffsetDateTime,
}

pub struct CardInfo {
    pub card: Card,
    pub artist: Artist,
    pub image: Image,
    pub illustration: Option<Illustration>,
    pub set: Set,
    pub rule: Rule,
    pub legality: Legality,
    pub price: Price,
}
```

- [ ] **Step 2: Update `ports/source.rs` to return `Vec<CardInfo>`**

```rust
use async_trait::async_trait;
use crate::ports::storage::{CardInfo, Set};

#[async_trait]
pub trait CardSource {
    async fn get_recent_sets(&self) -> Vec<Set>;

    /// Fetches cards for sets where the stored card count doesn't match the expected volume.
    /// Each entry is a `(Set, u32)` pair where the `u32` is the known card count for that set.
    /// Only sets with a volume mismatch are queried — up-to-date sets are skipped.
    async fn fetch_cards_for_outdated_sets(&self, sets: &[(Set, u32)]) -> Vec<CardInfo>;
}
```

- [ ] **Step 3: Verify it compiles (psql adapter will fail — that's expected)**

```bash
cargo build -p spoilers_upload 2>&1 | grep "error\[" | head -20
```

Expected: errors only in `psql/mod.rs` (missing `upsert_cards` impl) and `scryfall/mod.rs` (wrong return type). No errors in `ports/`.

- [ ] **Step 4: Commit**

```bash
git add packages/spoilers_upload/src/ports/storage.rs packages/spoilers_upload/src/ports/source.rs
git commit -m "feat(spoilers_upload): define storage port structs and update CardSource return type"
```

---

## Task 2: `increment_uuid` utility

**Files:**
- Create: `packages/spoilers_upload/src/adapters/services/scryfall/utils/mod.rs`
- Create: `packages/spoilers_upload/src/adapters/services/scryfall/utils/uuid.rs`
- Modify: `packages/spoilers_upload/src/adapters/services/scryfall/mod.rs` (add `pub mod utils;`)

- [ ] **Step 1: Write failing test in `utils/uuid.rs`**

Create `packages/spoilers_upload/src/adapters/services/scryfall/utils/uuid.rs`:

```rust
use uuid::Uuid;

pub fn increment_uuid(id: Uuid) -> Uuid {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_basic() {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        assert_eq!(
            increment_uuid(id),
            Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
        );
    }

    #[test]
    fn test_increment_realistic_uuid() {
        let id = Uuid::parse_str("0000419b-0bba-4488-8f7a-6194544ce91e").unwrap();
        assert_eq!(
            increment_uuid(id),
            Uuid::parse_str("0000419b-0bba-4488-8f7a-6194544ce91f").unwrap()
        );
    }

    #[test]
    fn test_increment_wraps_on_overflow() {
        let id = Uuid::from_u128(u128::MAX);
        assert_eq!(increment_uuid(id), Uuid::from_u128(0));
    }
}
```

Create `packages/spoilers_upload/src/adapters/services/scryfall/utils/mod.rs`:

```rust
pub mod image;
pub mod uuid;
```

Add to `packages/spoilers_upload/src/adapters/services/scryfall/mod.rs` (top, after existing `mod data;`):

```rust
pub mod utils;
```

- [ ] **Step 2: Run test to confirm it fails**

```bash
cargo test -p spoilers_upload increment 2>&1 | tail -10
```

Expected: `FAILED` with `not yet implemented`

- [ ] **Step 3: Implement `increment_uuid`**

Replace `todo!()` in `uuid.rs`:

```rust
pub fn increment_uuid(id: Uuid) -> Uuid {
    Uuid::from_u128(id.as_u128().wrapping_add(1))
}
```

- [ ] **Step 4: Run tests to confirm they pass**

```bash
cargo test -p spoilers_upload increment 2>&1 | tail -10
```

Expected: `3 passed`

- [ ] **Step 5: Commit**

```bash
git add packages/spoilers_upload/src/adapters/services/scryfall/utils/
git add packages/spoilers_upload/src/adapters/services/scryfall/mod.rs
git commit -m "feat(spoilers_upload): add increment_uuid utility"
```

---

## Task 3: `parse_image_id` utility

**Files:**
- Create: `packages/spoilers_upload/src/adapters/services/scryfall/utils/image.rs`
- Modify: `packages/spoilers_upload/Cargo.toml`

The Scryfall CDN URL format is:
`https://cards.scryfall.io/{png|art_crop}/{front|back}/{hex}/{hex}/{UUID}.{png|jpg}?{timestamp}`

For `back` URLs the extracted UUID is incremented (same logic as `increment_uuid`).

- [ ] **Step 1: Add `regex` to `Cargo.toml`**

In `packages/spoilers_upload/Cargo.toml`, add under `[dependencies]`:

```toml
regex = "1"
```

- [ ] **Step 2: Write failing tests in `utils/image.rs`**

Create `packages/spoilers_upload/src/adapters/services/scryfall/utils/image.rs`:

```rust
use std::sync::LazyLock;
use regex::Regex;
use uuid::Uuid;
use crate::adapters::services::scryfall::utils::uuid::increment_uuid;

static IMAGE_URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)https://cards\.scryfall\.io/(?:png|art_crop)/(front|back)/[0-9a-f]/[0-9a-f]/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})\.(?:png|jpg)\?\d+"
    ).unwrap()
});

pub fn parse_image_id(url: &str) -> Option<Uuid> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRONT_PNG: &str = "https://cards.scryfall.io/png/front/0/0/0000419b-0bba-4488-8f7a-6194544ce91e.png?1721427487";
    const BACK_PNG: &str  = "https://cards.scryfall.io/png/back/0/0/0000419b-0bba-4488-8f7a-6194544ce91e.png?1721427487";
    const ART_CROP: &str  = "https://cards.scryfall.io/art_crop/front/0/0/0000419b-0bba-4488-8f7a-6194544ce91e.jpg?1721427487";

    #[test]
    fn test_front_png_returns_uuid_unchanged() {
        let id = parse_image_id(FRONT_PNG).unwrap();
        assert_eq!(id, Uuid::parse_str("0000419b-0bba-4488-8f7a-6194544ce91e").unwrap());
    }

    #[test]
    fn test_back_png_returns_incremented_uuid() {
        let id = parse_image_id(BACK_PNG).unwrap();
        assert_eq!(id, Uuid::parse_str("0000419b-0bba-4488-8f7a-6194544ce91f").unwrap());
    }

    #[test]
    fn test_art_crop_front_returns_uuid_unchanged() {
        let id = parse_image_id(ART_CROP).unwrap();
        assert_eq!(id, Uuid::parse_str("0000419b-0bba-4488-8f7a-6194544ce91e").unwrap());
    }

    #[test]
    fn test_invalid_url_returns_none() {
        assert!(parse_image_id("https://example.com/not-scryfall").is_none());
    }

    #[test]
    fn test_empty_string_returns_none() {
        assert!(parse_image_id("").is_none());
    }
}
```

- [ ] **Step 3: Run tests to confirm they fail**

```bash
cargo test -p spoilers_upload parse_image_id 2>&1 | tail -10
```

Expected: `FAILED` with `not yet implemented`

- [ ] **Step 4: Implement `parse_image_id`**

Replace `todo!()`:

```rust
pub fn parse_image_id(url: &str) -> Option<Uuid> {
    let caps = IMAGE_URL_RE.captures(url)?;
    let side = caps.get(1)?.as_str();
    let uuid_str = caps.get(2)?.as_str();
    let id = Uuid::parse_str(uuid_str).ok()?;

    if side.eq_ignore_ascii_case("back") {
        Some(increment_uuid(id))
    } else {
        Some(id)
    }
}
```

- [ ] **Step 5: Run tests to confirm they pass**

```bash
cargo test -p spoilers_upload parse_image_id 2>&1 | tail -10
```

Expected: `5 passed`

- [ ] **Step 6: Commit**

```bash
git add packages/spoilers_upload/src/adapters/services/scryfall/utils/image.rs
git add packages/spoilers_upload/Cargo.toml Cargo.lock
git commit -m "feat(spoilers_upload): add parse_image_id utility"
```

---

## Task 4: Single-face conversion

**Files:**
- Modify: `packages/spoilers_upload/src/adapters/services/scryfall/data/card.rs`

Add imports and implement `into_storage_records` for the single-face path only. DFC is handled in Task 5.

- [ ] **Step 1: Write failing tests**

Add this `#[cfg(test)]` block to the bottom of `card.rs`:

```rust
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
                Uuid::parse_str("22ab27e3-6476-48f1-a9f7-9a9e86339030").unwrap()
            ]),
            illustration_id: Some(
                Uuid::parse_str("fb2b1ca2-7440-48c2-81c8-84da0a45a626").unwrap()
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
            type_line: Some("Basic Land — Forest".to_string()),
            oracle_text: Some("({T}: Add {G}.)".to_string()),
            colors: Some(vec![]),
            keywords: Some(vec![]),
            produced_mana: Some(vec!["G".to_string()]),
            rulings_uri: Some("https://api.scryfall.com/cards/0000419b-0bba-4488-8f7a-6194544ce91e/rulings".to_string()),
            prices: make_prices(),
            card_faces: None,
        }
    }

    #[test]
    fn test_single_face_produces_one_record() {
        let card = make_single_face_card();
        let records = card.into_storage_records().unwrap();
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn test_single_face_card_fields() {
        let card = make_single_face_card();
        let records = card.into_storage_records().unwrap();
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
        let card = make_single_face_card();
        let records = card.into_storage_records().unwrap();
        let price = &records[0].price;
        assert_eq!(price.usd, Some(0.35));
        assert_eq!(price.usd_foil, Some(0.40));
        assert_eq!(price.usd_etched, None);
        assert_eq!(price.euro, Some(0.24));
    }

    #[test]
    fn test_single_face_artist() {
        let card = make_single_face_card();
        let records = card.into_storage_records().unwrap();
        let artist = &records[0].artist;
        assert_eq!(artist.name, "David Robert Hovey");
        assert_eq!(
            artist.id,
            Uuid::parse_str("22ab27e3-6476-48f1-a9f7-9a9e86339030").unwrap()
        );
    }
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test -p spoilers_upload single_face 2>&1 | tail -15
```

Expected: compile error — `into_storage_records` does not exist yet.

- [ ] **Step 3: Add imports and implement single-face conversion**

Replace the full content of `card.rs` with:

```rust
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use time::serde::format_description;
use uuid::{uuid, Uuid};
use crate::adapters::services::scryfall::utils::image::parse_image_id;
use crate::adapters::services::scryfall::utils::uuid::increment_uuid;
use crate::ports::storage::{
    Artist, Card, CardInfo, Illustration, Image, Legality, Price, Rule, Set,
};

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
            Self::produce_matching_name_records(front, back, self)
        } else {
            Self::produce_dual_face_records(front, back, self)
        }
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
            usd_etched: self.prices.usd_etched.as_deref().and_then(|s| s.parse().ok()),
            euro: self.prices.eur.as_deref().and_then(|s| s.parse().ok()),
            euro_foil: self.prices.eur_foil.as_deref().and_then(|s| s.parse().ok()),
            tix: self.prices.tix.as_deref().and_then(|s| s.parse().ok()),
            updated_time: OffsetDateTime::now_utc(),
        }
    }

    fn into_single_face_record(self) -> Option<CardInfo> {
        let image_uris = self.image_uris.as_ref()?;
        let png_url = image_uris.png.as_deref()?;
        let image_id = parse_image_id(png_url)?;
        let art_crop = image_uris.art_crop.clone();
        let png_url = png_url.to_string();
        let oracle_id = self.oracle_id?;

        let image = Image { id: image_id, scryfall_url: png_url };

        let illustration = self.illustration_id
            .zip(art_crop)
            .map(|(id, url)| Illustration { id, scryfall_url: url });

        let artist_name = self.artist.as_deref().unwrap_or(ANONYMOUS_ARTIST);
        let artist = Artist {
            id: self.artist_ids
                .as_ref()
                .and_then(|ids| ids.first().copied())
                .unwrap_or(ANONYMOUS_ARTIST_ID),
            name: artist_name.to_string(),
            normalised_name: contracts::normalise::normalise(artist_name),
        };

        let card = Card {
            id: self.id,
            oracle_id,
            name: self.name.clone(),
            normalised_name: contracts::normalise::normalise(&self.name),
            scryfall_url: self.scryfall_uri.clone(),
            flavour_text: self.flavor_text,
            release_date: self.released_at,
            reserved: self.reserved,
            rarity: self.rarity,
            artist_id: artist.id,
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

        let legality = self.build_legality(oracle_id);
        let set = self.build_set();
        let price = self.build_price(self.id);

        Some(CardInfo { card, artist, image, illustration, set, rule, legality, price })
    }

    // DFC helpers added in Task 5
}

#[cfg(test)]
mod tests {
    // ... (paste test block from Step 1 here)
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p spoilers_upload single_face 2>&1 | tail -15
```

Expected: all single-face tests pass. DFC tests don't exist yet.

- [ ] **Step 5: Commit**

```bash
git add packages/spoilers_upload/src/adapters/services/scryfall/data/card.rs
git commit -m "feat(spoilers_upload): implement single-face ScryfallCard conversion"
```

---

## Task 5: Dual-face card conversion

**Files:**
- Modify: `packages/spoilers_upload/src/adapters/services/scryfall/data/card.rs`

- [ ] **Step 1: Write failing DFC tests**

Add these tests to the existing `#[cfg(test)]` block in `card.rs`:

```rust
fn make_card_face(name: &str, oracle_id: Option<&str>) -> CardFace {
    CardFace {
        name: name.to_string(),
        oracle_id: oracle_id.map(|s| Uuid::parse_str(s).unwrap()),
        mana_cost: Some("{1}{U}".to_string()),
        type_line: Some("Creature — Human".to_string()),
        oracle_text: Some("Flying".to_string()),
        colors: Some(vec!["U".to_string()]),
        power: Some("2".to_string()),
        toughness: Some("1".to_string()),
        loyalty: None,
        defense: None,
        flavor_text: None,
        artist: Some("Artist Name".to_string()),
        artist_ids: Some(vec![
            Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()
        ]),
        illustration_id: Some(
            Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap()
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
    card.image_uris = None; // DFCs have image_uris on faces, not top-level
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

#[test]
fn test_dfc_different_names_produces_two_records() {
    let card = make_dfc_card();
    let records = card.into_storage_records().unwrap();
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
    let card = make_dfc_card();
    let records = card.into_storage_records().unwrap();
    assert_ne!(records[0].card.oracle_id, records[1].card.oracle_id);
    assert_eq!(
        records[1].card.oracle_id,
        increment_uuid(records[0].card.oracle_id)
    );
}

#[test]
fn test_dfc_front_has_back_as_backside() {
    let card = make_dfc_card();
    let records = card.into_storage_records().unwrap();
    assert_eq!(records[0].card.backside_id, Some(records[1].card.id));
    assert_eq!(records[1].card.backside_id, Some(records[0].card.id));
}

#[test]
fn test_matching_name_dfc_shares_oracle_id() {
    let card = make_matching_name_dfc();
    let records = card.into_storage_records().unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].card.oracle_id, records[1].card.oracle_id);
}

#[test]
fn test_dfc_face_name_used_not_card_name() {
    let card = make_dfc_card();
    let records = card.into_storage_records().unwrap();
    assert_eq!(records[0].card.name, "Front Face");
    assert_eq!(records[1].card.name, "Back Face");
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test -p spoilers_upload dfc 2>&1 | tail -15
```

Expected: compile error — `produce_dual_face_records` / `produce_matching_name_records` not defined.

- [ ] **Step 3: Implement DFC helpers in `card.rs`**

Add these methods to the `impl ScryfallCard` block (replacing the `// DFC helpers added in Task 5` comment):

```rust
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

    let image = Image { id: image_id, scryfall_url: png_url };

    let illustration_id = face.illustration_id.or(card.illustration_id);
    let illustration = illustration_id
        .zip(art_crop)
        .map(|(id, url)| Illustration { id, scryfall_url: url });

    let artist_name = face.artist.as_deref()
        .or(card.artist.as_deref())
        .unwrap_or(ANONYMOUS_ARTIST);
    let artist = Artist {
        id: face.artist_ids.as_ref()
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

    Some(CardInfo { card: card_model, artist, image, illustration, set, rule, legality, price })
}

fn produce_dual_face_records(front: CardFace, back: CardFace, card: ScryfallCard) -> Option<Vec<CardInfo>> {
    let back_id = increment_uuid(card.id);
    let front_oracle_id = front.oracle_id.or(card.oracle_id)?;
    let back_oracle_id = increment_uuid(front_oracle_id);

    let front_record = Self::produce_face_record(front, &card, card.id, front_oracle_id, back_id)?;
    let back_record = Self::produce_face_record(back, &card, back_id, back_oracle_id, card.id)?;

    Some(vec![front_record, back_record])
}

fn produce_matching_name_records(front: CardFace, back: CardFace, card: ScryfallCard) -> Option<Vec<CardInfo>> {
    let back_id = increment_uuid(card.id);
    let oracle_id = front.oracle_id.or(card.oracle_id)?;

    let front_record = Self::produce_face_record(front, &card, card.id, oracle_id, back_id)?;
    let back_record = Self::produce_face_record(back, &card, back_id, oracle_id, card.id)?;

    Some(vec![front_record, back_record])
}
```

- [ ] **Step 4: Run all card.rs tests**

```bash
cargo test -p spoilers_upload 2>&1 | tail -20
```

Expected: all tests pass (single-face + DFC).

- [ ] **Step 5: Commit**

```bash
git add packages/spoilers_upload/src/adapters/services/scryfall/data/card.rs
git commit -m "feat(spoilers_upload): implement dual-face ScryfallCard conversion"
```

---

## Task 6: Wire conversion into scryfall adapter

**Files:**
- Modify: `packages/spoilers_upload/src/adapters/services/scryfall/mod.rs`

- [ ] **Step 1: Update `scryfall/mod.rs`**

Replace the full file with:

```rust
mod data;
pub mod utils;

use crate::adapters::services::scryfall::data::ScryfallData;
use crate::adapters::services::scryfall::data::card::ScryfallCard;
use crate::adapters::services::scryfall::data::set::ScryfallSet;
use crate::ports::source::CardSource;
use crate::ports::storage::{CardInfo, Set};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::env;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

struct ScryfallResponse<T> {
    scryfall_data: ScryfallData<T>,
    duration: Duration,
}

#[derive(Default)]
pub struct Scryfall {
    base_url: String,
    client: Client,
    sets: RwLock<HashMap<Uuid, ScryfallSet>>,
}

impl Scryfall {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(env::var("USER_AGENT").expect("USER_AGENT wasn't in env vars"))
            .build()
            .expect("Failure to creating reqwest client");

        Self {
            base_url: "https://api.scryfall.com".into(),
            client,
            ..Self::default()
        }
    }

    async fn recent_sets(&self) -> Vec<ScryfallSet> {
        let url = format!("{}/sets", self.base_url);
        let response = self.get::<ScryfallSet>(&url).await;

        let today = time::OffsetDateTime::now_utc().date();
        let threshold = today - time::Duration::days(7);

        response.scryfall_data.data.into_iter().filter_map(|set| {
            if set.released_at >= threshold && set.card_count > 0 {
                Some(set)
            } else {
                None
            }
        }).collect()
    }

    async fn get<T>(&self, url: &str) -> ScryfallResponse<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let start = Instant::now();
        let scryfall_data = self.client
            .get(url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        ScryfallResponse { scryfall_data, duration: start.elapsed() }
    }
}

#[async_trait]
impl CardSource for Scryfall {
    async fn get_recent_sets(&self) -> Vec<Set> {
        if !self.sets.read().await.is_empty() {
            return self.sets.read().await.iter().map(|(_, set)| set.into()).collect();
        }

        let sets = self.recent_sets().await;
        self.sets.write().await.extend(sets.into_iter().map(|set| (set.id, set.into())));

        self.sets.read().await.iter().map(|(_, set)| set.into()).collect()
    }

    async fn fetch_cards_for_outdated_sets(&self, sets: &[(Set, u32)]) -> Vec<CardInfo> {
        let mut scryfall_cards: Vec<ScryfallCard> = Vec::new();

        for (set, volume) in sets {
            if *volume == 0 {
                continue;
            }

            let is_outdated = self.sets.read().await
                .get(&set.id)
                .map_or(false, |s| s.card_count != *volume);

            if !is_outdated {
                continue;
            }

            let mut url = Some(format!(
                "{}/cards/search?q=e:{}",
                self.base_url, set.abbreviation
            ));
            while let Some(next_page) = url {
                let response = self.get(&next_page).await;
                scryfall_cards.extend(response.scryfall_data.data);
                url = response.scryfall_data.next_page;
                let sleep_time = Duration::from_millis(500).saturating_sub(response.duration);
                tokio::time::sleep(sleep_time).await;
            }
        }

        scryfall_cards
            .into_iter()
            .filter_map(|card| card.into_storage_records())
            .flatten()
            .collect()
    }
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build -p spoilers_upload 2>&1 | grep "error\[" | head -10
```

Expected: errors only in `psql/mod.rs` (missing `upsert_cards`). No errors in scryfall adapter.

- [ ] **Step 3: Commit**

```bash
git add packages/spoilers_upload/src/adapters/services/scryfall/mod.rs
git commit -m "feat(spoilers_upload): wire CardInfo conversion into scryfall fetch"
```

---

## Task 7: Implement `upsert_cards` in psql adapter

**Files:**
- Modify: `packages/spoilers_upload/src/adapters/services/psql/mod.rs`

Inserts in FK-safe order: artist → image → illustration → set → rule → legality → card → price.
`ON CONFLICT DO NOTHING` for immutable tables (artist, illustration, set).
`ON CONFLICT DO UPDATE` for mutable tables (image, rule, legality, card, price) to reflect changes.

- [ ] **Step 1: Replace `psql/mod.rs` with full implementation**

```rust
use std::env;
use async_trait::async_trait;
use futures::future;
use sqlx::{Pool, Row};
use sqlx::postgres::PgPoolOptions;
use crate::ports::storage::{CardInfo, Set, Storage};

pub struct Postgres {
    pool: Pool<sqlx::Postgres>,
}

impl Postgres {
    pub async fn create() -> Self {
        let user = env::var("POSTGRES_USER").expect("POSTGRES_USER wasn't in env vars");
        let password = env::var("POSTGRES_PW").expect("POSTGRES_PW wasn't in env vars");
        let db = env::var("POSTGRES_DB").expect("POSTGRES_DB wasn't in env vars");
        let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost:5432".to_string());
        let uri = format!("postgresql://{user}:{password}@{host}/{db}");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&uri)
            .await
            .expect("Failed Postgres connection");

        Self { pool }
    }

    async fn get_set_volume(&self, set: &Set) -> u32 {
        match sqlx::query("select count(*) from set where id = $1")
            .bind(set.id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(result) => result.try_get::<i64, &str>("count").unwrap_or(0) as u32,
            Err(why) => {
                log::warn!("Failed to fetch volume: {}", why);
                0
            }
        }
    }

    async fn upsert_card_info(&self, info: CardInfo) {
        let a = &info.artist;
        if let Err(e) = sqlx::query(
            "INSERT INTO artist (id, name, normalised_name) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
        )
        .bind(a.id)
        .bind(&a.name)
        .bind(&a.normalised_name)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert artist {}: {}", a.id, e);
        }

        let img = &info.image;
        if let Err(e) = sqlx::query(
            "INSERT INTO image (id, scryfall_url) VALUES ($1, $2) \
             ON CONFLICT (id) DO UPDATE SET scryfall_url = EXCLUDED.scryfall_url \
             WHERE image.scryfall_url IS DISTINCT FROM EXCLUDED.scryfall_url"
        )
        .bind(img.id)
        .bind(&img.scryfall_url)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert image {}: {}", img.id, e);
        }

        if let Some(ill) = &info.illustration {
            if let Err(e) = sqlx::query(
                "INSERT INTO illustration (id, scryfall_url) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(ill.id)
            .bind(&ill.scryfall_url)
            .execute(&self.pool)
            .await
            {
                log::warn!("Failed to upsert illustration {}: {}", ill.id, e);
            }
        }

        let s = &info.set;
        if let Err(e) = sqlx::query(
            "INSERT INTO set (id, name, normalised_name, abbreviation) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING"
        )
        .bind(s.id)
        .bind(&s.name)
        .bind(&s.normalised_name)
        .bind(&s.abbreviation)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert set {}: {}", s.id, e);
        }

        let r = &info.rule;
        if let Err(e) = sqlx::query(
            "INSERT INTO rule \
             (id, colour_identity, mana_cost, cmc, power, toughness, loyalty, defence, \
              type_line, oracle_text, colours, keywords, produced_mana, rulings_url) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) \
             ON CONFLICT(id) DO UPDATE SET \
               colour_identity = EXCLUDED.colour_identity, mana_cost = EXCLUDED.mana_cost, \
               cmc = EXCLUDED.cmc, power = EXCLUDED.power, toughness = EXCLUDED.toughness, \
               loyalty = EXCLUDED.loyalty, defence = EXCLUDED.defence, \
               type_line = EXCLUDED.type_line, oracle_text = EXCLUDED.oracle_text, \
               colours = EXCLUDED.colours, keywords = EXCLUDED.keywords, \
               produced_mana = EXCLUDED.produced_mana, rulings_url = EXCLUDED.rulings_url"
        )
        .bind(r.id)
        .bind(&r.colour_identity)
        .bind(&r.mana_cost)
        .bind(r.cmc)
        .bind(&r.power)
        .bind(&r.toughness)
        .bind(&r.loyalty)
        .bind(&r.defence)
        .bind(&r.type_line)
        .bind(&r.oracle_text)
        .bind(&r.colours)
        .bind(&r.keywords)
        .bind(&r.produced_mana)
        .bind(&r.rulings_url)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert rule {}: {}", r.id, e);
        }

        let leg = &info.legality;
        if let Err(e) = sqlx::query(
            "INSERT INTO legality \
             (id, alchemy, brawl, commander, duel, future, gladiator, historic, legacy, modern, \
              oathbreaker, oldschool, pauper, paupercommander, penny, pioneer, predh, premodern, \
              standard, standardbrawl, timeless, vintage, game_changer) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, \
                     $17, $18, $19, $20, $21, $22, $23) \
             ON CONFLICT(id) DO UPDATE SET \
               alchemy = EXCLUDED.alchemy, brawl = EXCLUDED.brawl, \
               commander = EXCLUDED.commander, duel = EXCLUDED.duel, \
               future = EXCLUDED.future, gladiator = EXCLUDED.gladiator, \
               historic = EXCLUDED.historic, legacy = EXCLUDED.legacy, \
               modern = EXCLUDED.modern, oathbreaker = EXCLUDED.oathbreaker, \
               oldschool = EXCLUDED.oldschool, pauper = EXCLUDED.pauper, \
               paupercommander = EXCLUDED.paupercommander, penny = EXCLUDED.penny, \
               pioneer = EXCLUDED.pioneer, predh = EXCLUDED.predh, \
               premodern = EXCLUDED.premodern, standard = EXCLUDED.standard, \
               standardbrawl = EXCLUDED.standardbrawl, timeless = EXCLUDED.timeless, \
               vintage = EXCLUDED.vintage, game_changer = EXCLUDED.game_changer"
        )
        .bind(leg.id)
        .bind(&leg.alchemy)
        .bind(&leg.brawl)
        .bind(&leg.commander)
        .bind(&leg.duel)
        .bind(&leg.future)
        .bind(&leg.gladiator)
        .bind(&leg.historic)
        .bind(&leg.legacy)
        .bind(&leg.modern)
        .bind(&leg.oathbreaker)
        .bind(&leg.oldschool)
        .bind(&leg.pauper)
        .bind(&leg.paupercommander)
        .bind(&leg.penny)
        .bind(&leg.pioneer)
        .bind(&leg.predh)
        .bind(&leg.premodern)
        .bind(&leg.standard)
        .bind(&leg.standardbrawl)
        .bind(&leg.timeless)
        .bind(&leg.vintage)
        .bind(leg.game_changer)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert legality {}: {}", leg.id, e);
        }

        let c = &info.card;
        if let Err(e) = sqlx::query(
            "INSERT INTO card \
             (id, oracle_id, name, normalised_name, scryfall_url, flavour_text, release_date, \
              reserved, rarity, artist_id, image_id, illustration_id, set_id, backside_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) \
             ON CONFLICT (id) DO UPDATE SET \
               normalised_name = EXCLUDED.normalised_name, \
               scryfall_url = EXCLUDED.scryfall_url, \
               reserved = EXCLUDED.reserved, \
               oracle_id = EXCLUDED.oracle_id \
             WHERE (card.normalised_name IS DISTINCT FROM EXCLUDED.normalised_name OR \
                    card.scryfall_url    IS DISTINCT FROM EXCLUDED.scryfall_url    OR \
                    card.reserved        IS DISTINCT FROM EXCLUDED.reserved        OR \
                    card.oracle_id       IS DISTINCT FROM EXCLUDED.oracle_id)"
        )
        .bind(c.id)
        .bind(c.oracle_id)
        .bind(&c.name)
        .bind(&c.normalised_name)
        .bind(&c.scryfall_url)
        .bind(&c.flavour_text)
        .bind(c.release_date)
        .bind(c.reserved)
        .bind(&c.rarity)
        .bind(c.artist_id)
        .bind(c.image_id)
        .bind(c.illustration_id)
        .bind(c.set_id)
        .bind(c.backside_id)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert card {}: {}", c.id, e);
        }

        let p = &info.price;
        if let Err(e) = sqlx::query(
            "INSERT INTO price (id, usd, usd_foil, usd_etched, euro, euro_foil, tix, updated_time) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (id) DO UPDATE SET \
               usd = EXCLUDED.usd, usd_foil = EXCLUDED.usd_foil, \
               usd_etched = EXCLUDED.usd_etched, euro = EXCLUDED.euro, \
               euro_foil = EXCLUDED.euro_foil, tix = EXCLUDED.tix, \
               updated_time = EXCLUDED.updated_time"
        )
        .bind(p.id)
        .bind(p.usd)
        .bind(p.usd_foil)
        .bind(p.usd_etched)
        .bind(p.euro)
        .bind(p.euro_foil)
        .bind(p.tix)
        .bind(p.updated_time)
        .execute(&self.pool)
        .await
        {
            log::warn!("Failed to upsert price {}: {}", p.id, e);
        }
    }
}

#[async_trait]
impl Storage for Postgres {
    async fn get_set_volumes(&self, sets: Vec<Set>) -> Vec<(Set, u32)> {
        future::join_all(
            sets.into_iter().map(|set| async {
                let volume = self.get_set_volume(&set).await;
                (set, volume)
            })
        ).await
    }

    async fn upsert_cards(&self, cards: Vec<CardInfo>) {
        for card_info in cards {
            self.upsert_card_info(card_info).await;
        }
    }
}
```

- [ ] **Step 2: Verify the full project compiles**

```bash
cargo build -p spoilers_upload 2>&1 | grep "error\[" | head -10
```

Expected: clean build (warnings OK).

- [ ] **Step 3: Run all tests**

```bash
cargo test -p spoilers_upload 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
git add packages/spoilers_upload/src/adapters/services/psql/mod.rs
git commit -m "feat(spoilers_upload): implement upsert_cards in postgres adapter"
```

---

## Task 8: Wire `upsert_cards` in `main.rs`

**Files:**
- Modify: `packages/spoilers_upload/src/main.rs`

- [ ] **Step 1: Update `main.rs`**

```rust
use crate::ports::storage::Storage;
use crate::ports::source::CardSource;
use crate::adapters::services::{card_storage_init, card_source_init};
#[cfg(feature = "local-dev")]
use dotenv::dotenv;

pub mod adapters;
pub mod ports;

#[tokio::main]
async fn main() {
    #[cfg(feature = "local-dev")]
    dotenv().ok();

    let source = card_source_init();
    let storage = card_storage_init().await;

    let sets = source.get_recent_sets().await;
    let set_volumes = storage.get_set_volumes(sets).await;
    let cards = source.fetch_cards_for_outdated_sets(&set_volumes).await;

    storage.upsert_cards(cards).await;
}
```

- [ ] **Step 2: Final build and test**

```bash
cargo build -p spoilers_upload 2>&1 | grep "error\[" | head -10
cargo test -p spoilers_upload 2>&1 | tail -10
```

Expected: clean build, all tests pass.

- [ ] **Step 3: Commit**

```bash
git add packages/spoilers_upload/src/main.rs
git commit -m "feat(spoilers_upload): wire upsert_cards into main loop"
```

---

## Self-Review

**Spec coverage:**
- ✅ Storage structs in `ports/storage.rs` with no `DB` prefix
- ✅ Utils in their own `scryfall/utils/` folder
- ✅ `Option`-based error handling (silent skip like Python)
- ✅ Single-face path
- ✅ DFC different-name path (incremented oracle_ids)
- ✅ DFC matching-name path (shared oracle_id)
- ✅ Field fallback chain: face field → top-level card field → default
- ✅ FK-safe insert order in psql adapter
- ✅ `upsert_cards` wired into `main.rs`

**Type consistency check:**
- `CardInfo` defined in Task 1, used identically in Tasks 4, 5, 6, 7
- `increment_uuid` defined in Task 2, imported via `crate::adapters::services::scryfall::utils::uuid::increment_uuid` in Tasks 3, 5
- `parse_image_id` defined in Task 3, imported the same way in Task 4
- `build_legality`, `build_set`, `build_price` defined and used within Task 4 — no Task 5 drift
- `produce_face_record` added in Task 5, called twice within same impl block ✅