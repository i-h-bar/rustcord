use contracts::card::Card;

pub fn create_description(card: &Card) -> String {
    let stat = match (card.power(), card.toughness(), card.loyalty(), card.defence()) {
        (Some(p), Some(t), _, _) => format!(" • ⚔️ {p}/{t}"),
        (_, _, Some(l), _)       => format!(" • 💠 {l}"),
        (_, _, _, Some(d))       => format!(" • 🛡️ {d}"),
        _                        => String::new(),
    };

    format!("{} • {}{}", card.type_line(), card.mana_cost(), stat)
}