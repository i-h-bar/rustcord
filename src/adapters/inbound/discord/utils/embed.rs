use crate::adapters::inbound::discord::utils::colours::get_colour_identity;
use crate::adapters::inbound::discord::utils::emoji::add_emoji;
use crate::adapters::inbound::discord::utils::italicise_reminder_text;
use crate::adapters::inbound::discord::utils::REGEX_COLLECTION;
use crate::domain::card::Card;
use regex::Captures;
use serenity::all::{CreateEmbed, CreateEmbedFooter};

pub fn create_game_embed(card: &Card, multiplier: usize, guesses: usize) -> CreateEmbed {
    let mut embed = CreateEmbed::default()
        .attachment(format!(
            "{}.png",
            card.front_illustration_id.unwrap_or_default()
        ))
        .title("????")
        .description("????")
        .footer(CreateEmbedFooter::new(format!("🖌️ - {}", card.artist)));

    if guesses > multiplier {
        let mana_cost = REGEX_COLLECTION
            .symbols
            .replace_all(&card.front_mana_cost, |cap: &Captures| add_emoji(cap));
        let title = format!("????        {mana_cost}");
        embed = embed
            .title(title)
            .colour(get_colour_identity(&card.front_colour_identity));
    }

    if guesses > multiplier * 2 {
        let stats = if let Some(power) = card.front_power.clone() {
            let toughness = card
                .front_toughness
                .clone()
                .unwrap_or_else(|| "0".to_string());
            format!("\n\n{power}/{toughness}")
        } else if let Some(loyalty) = card.front_loyalty.clone() {
            format!("\n\n{loyalty}")
        } else if let Some(defence) = card.front_defence.clone() {
            format!("\n\n{defence}")
        } else {
            String::new()
        };

        let front_rules_text = REGEX_COLLECTION
            .symbols
            .replace_all(&card.front_oracle_text, |cap: &Captures| add_emoji(cap));

        let front_oracle_text = italicise_reminder_text(&front_rules_text);

        embed = embed.description(format!(
            "{}\n\n{}{}",
            card.front_type_line, front_oracle_text, stats
        ));
    }

    embed
}

pub fn create_embed(card: Card) -> CreateEmbed {
    let stats = if let Some(power) = card.front_power {
        let toughness = card.front_toughness.unwrap_or_else(|| "0".to_string());
        format!("\n\n{power}/{toughness}")
    } else if let Some(loyalty) = card.front_loyalty {
        format!("\n\n{loyalty}")
    } else if let Some(defence) = card.front_defence {
        format!("\n\n{defence}")
    } else {
        String::new()
    };

    let front_oracle_text = REGEX_COLLECTION
        .symbols
        .replace_all(&card.front_oracle_text, |cap: &Captures| add_emoji(cap));
    let front_oracle_text = italicise_reminder_text(&front_oracle_text);

    let rules_text = format!("{}\n\n{}{}", card.front_type_line, front_oracle_text, stats);
    let mana_cost = REGEX_COLLECTION
        .symbols
        .replace_all(&card.front_mana_cost, |cap: &Captures| add_emoji(cap));
    let title = format!("{}        {}", card.front_name, mana_cost);

    CreateEmbed::default()
        .attachment(format!("{}.png", card.front_image_id))
        .url(card.front_scryfall_url)
        .title(title)
        .description(rules_text)
        .colour(get_colour_identity(&card.front_colour_identity))
        .footer(CreateEmbedFooter::new(format!("🖌️ - {}", card.artist)))
}
