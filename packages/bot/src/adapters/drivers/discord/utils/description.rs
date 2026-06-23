use contracts::card::Card;
use contracts::card_set::CardSet;

const LAND_TYPE_MAP: [(&str, &str); 5] = [
    ("Mountain", "⛰️"),
    ("Plains", "☀️"),
    ("Swamp", "💀"),
    ("Island", "💧"),
    ("Forest", "🌲"),
];

pub fn create_card_description(card: &Card) -> String {
    let stat = match (
        card.power(),
        card.toughness(),
        card.loyalty(),
        card.defence(),
    ) {
        (Some(p), Some(t), _, _) => format!(" • ⚔️ {p}/{t}"),
        (_, _, Some(l), _) => format!(" • 💠 {l}"),
        (_, _, _, Some(d)) => format!(" • 🛡️ {d}"),
        _ => String::new(),
    };

    let cost = match card.mana_cost() {
        "" => {
            let type_line = card.type_line();
            if type_line.starts_with("Land")
                || type_line.starts_with("Basic")
                || type_line.starts_with("Snow")
            {
                let mut base = String::from(" • ");
                let mut contained_type = false;
                for (land_type, emoji) in LAND_TYPE_MAP {
                    if type_line.contains(land_type) {
                        contained_type = true;
                        base.push_str(emoji);
                    }
                }

                if !contained_type {
                    base.push('🧭');
                }

                base
            } else {
                String::new()
            }
        }
        cost => format!(" • {cost}"),
    };

    format!("{}{}{}", card.type_line(), cost, stat)
}

pub fn create_set_description(set: &CardSet) -> String {
    format!("{} • {}", set.abbreviation(), set.release_date())
}
