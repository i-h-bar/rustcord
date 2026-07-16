use crate::colours::get_colour_identity;
use crate::emoji::add_emoji;
use crate::emoji_cache::get_emoji;
use crate::regex::REGEX_COLLECTION;
use crate::title::create_title;
use contracts::card::Card;
use regex::Captures;
use serenity::all::{CreateEmbed, CreateEmbedFooter};

#[must_use]
pub fn italicise_reminder_text(text: &str) -> String {
    REGEX_COLLECTION
        .reminder_text
        .replace_all(text, |cap: &Captures| format!("(*{}*)", &cap[1]))
        .to_string()
}

/// Builds the rich embed shared by `/search` and spoiler notifications:
/// title, set/type/rules text, colour-identity border, artist footer. Does
/// *not* set the embed's image — callers attach a local file (`bot`) or a
/// remote URL (`notifier`) afterward, since those two hosting strategies are
/// the only difference between the two use sites.
async fn build_embed(card: &Card) -> CreateEmbed {
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

    let oracle_text = add_emoji(card.oracle_text()).await;
    let oracle_text = italicise_reminder_text(&oracle_text);

    let set_emoji = match get_emoji(card.set_abbreviation()).await {
        Some(emoji) => format!(" <:{}:{}>", emoji.name, emoji.id),
        None => String::new(),
    };

    let rules_text = format!(
        "{}  {}\n\n{}\n\n{}{}",
        set_emoji,
        card.set_name(),
        card.type_line(),
        oracle_text,
        stats
    );
    let title = create_title(card).await;

    CreateEmbed::default()
        .url(card.url())
        .title(title)
        .description(rules_text)
        .colour(get_colour_identity(card.colour_identity()))
        .footer(CreateEmbedFooter::new(format!("🖌️ - {}", card.artist())))
}

/// Used by `bot`'s `/search`: attaches a local image file from `IMAGES_DIR`
/// (the caller is expected to also attach a `CreateAttachment` named
/// `{image_id}.png` to the outgoing message).
pub async fn create_embed(card: &Card) -> CreateEmbed {
    build_embed(card)
        .await
        .attachment(format!("{}.png", card.image_id()))
}

/// Used by `notifier`: sets the embed image directly to a remote URL (the
/// card image's own `scryfall_url`) instead of a local attachment, so the
/// scale-to-zero `notifier` pod never needs the card-image volume mounted.
pub async fn create_embed_with_image_url(card: &Card, image_url: &str) -> CreateEmbed {
    build_embed(card).await.image(image_url)
}
