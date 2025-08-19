use crate::domain::card::Card;
use crate::ports::clients::discord::utils::REGEX_COLLECTION;
use crate::ports::clients::discord::utils::colours::get_colour_identity;
use crate::ports::clients::discord::utils::emoji::add_emoji;
use crate::ports::clients::discord::utils::italicise_reminder_text;
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

pub fn create_embed(card: Card) -> (CreateEmbed, Option<CreateEmbed>) {
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

    let front = CreateEmbed::default()
        .attachment(format!("{}.png", card.front_image_id))
        .url(card.front_scryfall_url)
        .title(title)
        .description(rules_text)
        .colour(get_colour_identity(&card.front_colour_identity))
        .footer(CreateEmbedFooter::new(format!("🖌️ - {}", card.artist)));

    let back = if let Some(name) = card.back_name {
        let stats = if let Some(power) = card.back_power {
            let toughness = card.back_toughness.unwrap_or_else(|| "0".to_string());
            format!("\n\n{power}/{toughness}")
        } else if let Some(loyalty) = card.back_loyalty {
            format!("\n\n{loyalty}")
        } else if let Some(defence) = card.back_defence {
            format!("\n\n{defence}")
        } else {
            String::new()
        };
        let back_oracle_text = card.back_oracle_text.unwrap_or_default();
        let back_oracle_text = REGEX_COLLECTION
            .symbols
            .replace_all(&back_oracle_text, |cap: &Captures| add_emoji(cap));
        let back_oracle_text = italicise_reminder_text(&back_oracle_text);

        let back_rules_text = format!(
            "{}\n\n{}{}",
            card.back_type_line.unwrap_or_default(),
            back_oracle_text,
            stats
        );
        let title = if let Some(mana_cost) = card.back_mana_cost {
            let mana_cost = REGEX_COLLECTION
                .symbols
                .replace_all(&mana_cost, |cap: &Captures| add_emoji(cap));
            format!("{name}        {mana_cost}")
        } else {
            name
        };

        let url = card.back_scryfall_url.unwrap_or_default();
        Some(
            CreateEmbed::default()
                .attachment(format!("{}.png", card.back_image_id.unwrap_or_default()))
                .url(url)
                .title(title)
                .description(back_rules_text)
                .colour(get_colour_identity(
                    &card.back_colour_identity.unwrap_or_default(),
                ))
                .footer(CreateEmbedFooter::new(format!("🖌️ - {}", card.artist))),
        )
    } else {
        None
    };

    (front, back)
}
