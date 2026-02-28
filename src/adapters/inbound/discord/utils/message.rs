use crate::adapters::inbound::discord::components::interaction::{FLIP, PICK_PRINT_ID};
use crate::domain::card::Card;
use crate::domain::set::Set;
use serenity::all::{
    ButtonStyle, CreateActionRow, CreateButton, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};

pub fn build_set_dropdown(sets: Option<Vec<Set>>) -> Option<CreateActionRow> {
    if let Some(sets) = sets {
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

pub fn build_flip_button(card: &Card) -> Option<CreateActionRow> {
    if let Some(back_id) = card.back_id {
        let button = CreateButton::new(format!("{FLIP}{back_id}"))
            .label("🔁")
            .style(ButtonStyle::Secondary);
        Some(CreateActionRow::Buttons(vec![button]))
    } else {
        None
    }
}
