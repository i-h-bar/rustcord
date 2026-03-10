use crate::adapters::drivers::discord::utils::colours::get_colour_identity;
use crate::adapters::drivers::discord::utils::emoji::add_emoji;
use crate::adapters::drivers::discord::utils::italicise_reminder_text;
use crate::adapters::drivers::discord::utils::REGEX_COLLECTION;
use contracts::card::Card;
use regex::Captures;
use serenity::all::{CreateEmbed, CreateEmbedFooter};
use uuid::Uuid;

pub fn create_game_embed(card: &Card, multiplier: usize, guesses: usize) -> CreateEmbed {
    let mut embed = CreateEmbed::default()
        .attachment(format!(
            "{}.png",
            card.illustration_id().unwrap_or(&Uuid::default())
        ))
        .title("????")
        .description("????")
        .footer(CreateEmbedFooter::new(format!("🖌️ - {}", card.artist())));

    if guesses > multiplier {
        let mana_cost = REGEX_COLLECTION
            .symbols
            .replace_all(card.mana_cost(), |cap: &Captures| add_emoji(cap));
        let title = format!("????        {mana_cost}");
        embed = embed
            .title(title)
            .colour(get_colour_identity(card.colour_identity()));
    }

    if guesses > multiplier * 2 {
        let stats = if let Some(power) = card.power() {
            let toughness = card
                .toughness()
                .unwrap_or("0");
            format!("\n\n{power}/{toughness}")
        } else if let Some(loyalty) = card.loyalty() {
            format!("\n\n{loyalty}")
        } else if let Some(defence) = card.defence() {
            format!("\n\n{defence}")
        } else {
            String::new()
        };

        let rules_text = REGEX_COLLECTION
            .symbols
            .replace_all(card.oracle_text(), |cap: &Captures| add_emoji(cap));

        let oracle_text = italicise_reminder_text(&rules_text);

        embed = embed.description(format!(
            "{}\n\n{}{}",
            card.type_line(), oracle_text, stats
        ));
    }

    embed
}

pub fn create_embed(card: &Card) -> CreateEmbed {
    let stats = if let Some(power) = card.power() {
        let toughness = card.toughness().unwrap_or("0");
        format!("\n\n{power}/{toughness}")
    } else if let Some(loyalty) = card.loyalty() {
        format!("\n\n{loyalty}")
    } else if let Some(defence) = card.defence() {
        format!("\n\n{defence}")
    } else {
        String::new()
    };

    let oracle_text = REGEX_COLLECTION
        .symbols
        .replace_all(card.oracle_text(), |cap: &Captures| add_emoji(cap));
    let oracle_text = italicise_reminder_text(&oracle_text);

    let rules_text = format!("{}\n\n{}{}", card.type_line(), oracle_text, stats);
    let mana_cost = REGEX_COLLECTION
        .symbols
        .replace_all(card.mana_cost(), |cap: &Captures| add_emoji(cap));
    let title = format!("{}        {}", card.name(), mana_cost);

    CreateEmbed::default()
        .attachment(format!("{}.png", card.image_id()))
        .url(card.url())
        .title(title)
        .description(rules_text)
        .colour(get_colour_identity(card.colour_identity()))
        .footer(CreateEmbedFooter::new(format!("🖌️ - {}", card.artist())))
}
