use serenity::model::Colour;

pub fn get_colour_identity(colour_id: &[String]) -> Colour {
    let colour_id = colour_id.join("");
    let (r, g, b) = get_colour_num(&colour_id);

    Colour::from_rgb(r, g, b)
}

fn get_colour_num(colour: &str) -> (u8, u8, u8) {
    match colour {
        "R" => (220, 20, 60),
        "G" => (34, 139, 34),
        "U" => (0x04, 0x92, 0xC2),
        "W" => (240, 230, 210),
        "B" => (22, 13, 8),
        "UW" => (0x95, 0xB9, 0xDB),
        "BU" => (0x05, 0x01, 0x4A),
        "BR" => (0x46, 0, 0),
        "GR" => (0x89, 0x51, 0x29),
        "GW" => (0x98, 0xC3, 0x77),
        "BW" => (0x44, 0x44, 0x44),
        "RU" => (0x9D, 0x00, 0xFF),
        "BG" => (0x06, 0x40, 0x2B),
        "RW" => (0xD9, 0x54, 0x4D),
        "GU" => (0x00, 0xB3, 0xB3),
        "BUW" => (0x36, 0x55, 0x63),
        "BRU" => (0x34, 0x15, 0x39),
        "BGR" => (0x50, 0x37, 0x30),
        "GRW" => (0xE5, 0x9B, 0x5A),
        "GUW" => (0x81, 0xFF, 0xFF),
        "BGW" => (0x74, 0x93, 0x6A),
        "RUW" => (0xF9, 0x48, 0xED),
        "BGU" => (0x00, 0x41, 0x41),
        "BRW" => (0xA0, 0x6E, 0x69),
        "GRU" => (0x8F, 0x79, 0xA1),
        "BGRUW" => (0xFF, 0xFF, 0xFF),
        "" => (0xA9, 0xA9, 0xA9),
        _ => (0xF9, 0xC7, 0x4F),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mono_red_colour() {
        let colour = get_colour_identity(&["R".to_string()]);
        assert_eq!(colour.r(), 220);
        assert_eq!(colour.g(), 20);
        assert_eq!(colour.b(), 60);
    }

    #[test]
    fn test_mono_blue_colour() {
        let colour = get_colour_identity(&["U".to_string()]);
        assert_eq!(colour.r(), 0x04);
        assert_eq!(colour.g(), 0x92);
        assert_eq!(colour.b(), 0xC2);
    }

    #[test]
    fn test_azorius_colours() {
        // White-Blue (Azorius)
        let colour = get_colour_identity(&["U".to_string(), "W".to_string()]);
        assert_eq!(colour.r(), 0x95);
        assert_eq!(colour.g(), 0xB9);
        assert_eq!(colour.b(), 0xDB);
    }

    #[test]
    fn test_golgari_colours() {
        // Black-Green (Golgari)
        let colour = get_colour_identity(&["B".to_string(), "G".to_string()]);
        assert_eq!(colour.r(), 0x06);
        assert_eq!(colour.g(), 0x40);
        assert_eq!(colour.b(), 0x2B);
    }

    #[test]
    fn test_five_colour_card() {
        // WUBRG (all five colors)
        let colour = get_colour_identity(&[
            "B".to_string(),
            "G".to_string(),
            "R".to_string(),
            "U".to_string(),
            "W".to_string(),
        ]);
        // Should be white (all colors)
        assert_eq!(colour.r(), 0xFF);
        assert_eq!(colour.g(), 0xFF);
        assert_eq!(colour.b(), 0xFF);
    }

    #[test]
    fn test_colorless_card() {
        let colour = get_colour_identity(&[]);
        // Should be gray (colorless)
        assert_eq!(colour.r(), 0xA9);
        assert_eq!(colour.g(), 0xA9);
        assert_eq!(colour.b(), 0xA9);
    }

    #[test]
    fn test_unknown_colour_combination() {
        // Test that unknown combinations get the gold/multicolor default
        let colour = get_colour_identity(&["X".to_string(), "Y".to_string()]);
        assert_eq!(colour.r(), 0xF9);
        assert_eq!(colour.g(), 0xC7);
        assert_eq!(colour.b(), 0x4F);
    }

    #[test]
    fn test_bant_colours() {
        // White-Blue-Green (Bant)
        let colour = get_colour_identity(&["G".to_string(), "U".to_string(), "W".to_string()]);
        assert_eq!(colour.r(), 0x81);
        assert_eq!(colour.g(), 0xFF);
        assert_eq!(colour.b(), 0xFF);
    }

    #[test]
    fn test_colour_identity_joins_correctly() {
        // Test that the join operation works as expected
        let single = get_colour_identity(&["R".to_string()]);
        let double = get_colour_identity(&["G".to_string(), "R".to_string()]);

        // Single should be mono-red
        assert_eq!(single.r(), 220);

        // Double should be Gruul
        assert_eq!(double.r(), 0x89);
    }
}

