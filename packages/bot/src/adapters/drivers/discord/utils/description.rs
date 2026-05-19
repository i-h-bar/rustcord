use contracts::card::Card;
use contracts::set::Set;

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

    format!("{} • {}{}", card.type_line(), card.mana_cost(), stat)
}

pub fn create_set_description(set: &Set) -> String {
    format!("{} • {}", set.abbreviation(), set.release_date())
}
