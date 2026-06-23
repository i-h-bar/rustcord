use crate::adapters::services::scryfall::utils::uuid::increment_uuid;
use regex::Regex;
use std::sync::LazyLock;
use uuid::Uuid;

static IMAGE_URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)https://cards\.scryfall\.io/(?:png|art_crop)/(front|back)/[0-9a-f]/[0-9a-f]/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})\.(?:png|jpg)\?\d+"
    ).unwrap()
});

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

#[cfg(test)]
mod tests {
    use super::*;

    const FRONT_PNG: &str = "https://cards.scryfall.io/png/front/0/0/0000419b-0bba-4488-8f7a-6194544ce91e.png?1721427487";
    const BACK_PNG: &str = "https://cards.scryfall.io/png/back/0/0/0000419b-0bba-4488-8f7a-6194544ce91e.png?1721427487";
    const ART_CROP: &str = "https://cards.scryfall.io/art_crop/front/0/0/0000419b-0bba-4488-8f7a-6194544ce91e.jpg?1721427487";

    #[test]
    fn test_front_png_returns_uuid_unchanged() {
        let id = parse_image_id(FRONT_PNG).unwrap();
        assert_eq!(
            id,
            Uuid::parse_str("0000419b-0bba-4488-8f7a-6194544ce91e").unwrap()
        );
    }

    #[test]
    fn test_back_png_returns_incremented_uuid() {
        let id = parse_image_id(BACK_PNG).unwrap();
        assert_eq!(
            id,
            Uuid::parse_str("0000419b-0bba-4488-8f7a-6194544ce91f").unwrap()
        );
    }

    #[test]
    fn test_art_crop_front_returns_uuid_unchanged() {
        let id = parse_image_id(ART_CROP).unwrap();
        assert_eq!(
            id,
            Uuid::parse_str("0000419b-0bba-4488-8f7a-6194544ce91e").unwrap()
        );
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
