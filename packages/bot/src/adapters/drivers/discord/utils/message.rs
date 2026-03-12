use crate::adapters::drivers::discord::components::interaction::{FLIP, PICK_PRINT_ID, SIMILAR_ID};
use contracts::card::Card;
use contracts::set::Set;
use serenity::all::{
    ButtonStyle, CreateActionRow, CreateButton, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};

pub fn build_set_dropdown(sets: Option<&Vec<Set>>) -> Option<CreateActionRow> {
    if let Some(sets) = sets {
        if sets.is_empty() {
            return None;
        }

        if sets.len() > 1 {
            let options: Vec<CreateSelectMenuOption> = sets
                .iter()
                .take(25) // Discord's hard limit
                .map(|s| CreateSelectMenuOption::new(s.name(), s.card_id().to_string()))
                .collect();
            let menu =
                CreateSelectMenu::new(PICK_PRINT_ID, CreateSelectMenuKind::String { options })
                    .placeholder("Select a print...");
            return Some(CreateActionRow::SelectMenu(menu));
        }
    }

    None
}

pub fn build_similar_dropdown(similar: Option<&Vec<Card>>) -> Option<CreateActionRow> {
    if let Some(cards) = similar {
        if cards.is_empty() {
            return None;
        }
        let options: Vec<CreateSelectMenuOption> = cards
            .iter()
            .take(25) // Discord's hard limit
            .map(|c| CreateSelectMenuOption::new(c.name(), c.id().to_string()))
            .collect();
        let menu = CreateSelectMenu::new(SIMILAR_ID, CreateSelectMenuKind::String { options })
            .placeholder("Similar cards...");
        return Some(CreateActionRow::SelectMenu(menu));
    }

    None
}

pub fn build_flip_button(card: &Card) -> Option<CreateActionRow> {
    if let Some(back_id) = card.back_id() {
        let button = CreateButton::new(format!("{FLIP}{back_id}"))
            .label("🔁")
            .style(ButtonStyle::Secondary);
        Some(CreateActionRow::Buttons(vec![button]))
    } else {
        None
    }
}
