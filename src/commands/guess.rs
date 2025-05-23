use crate::game::state;
use crate::mtg::images::ImageFetcher;
use crate::utils::parse::{ParseError, ResolveOption};
use crate::utils::{fuzzy, normalise, parse};
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, MessageBuilder, ResolvedValue,
};

pub async fn run(ctx: &Context, interaction: &CommandInteraction) {
    let Options { guess } = match parse::options(interaction.data.options()) {
        Ok(value) => value,
        Err(err) => {
            log::warn!("Failed to parse guess: {}", err);
            return;
        }
    };

    let Some(image_fetcher) = ImageFetcher::get() else {
        log::warn!("couldn't get image fetcher");
        return;
    };

    let Some(game_state) = state::fetch(ctx, interaction).await else {
        return;
    };

    if fuzzy::jaro_winkler(&normalise(&guess), &game_state.card().front_normalised_name) > 0.75 {
        let (Some(image), _) = image_fetcher.fetch(game_state.card()).await else {
            log::warn!("couldn't fetch image");
            return;
        };

        let number_of_guesses = game_state.number_of_guesses();
        let guess_plural = if number_of_guesses > 1 {
            "guesses"
        } else {
            "guess"
        };

        let message = MessageBuilder::new()
            .mention(&interaction.user)
            .push(format!(
                " has won after {} {}!",
                number_of_guesses, guess_plural
            ))
            .build();

        let embed = game_state.convert_to_embed();

        let response = CreateInteractionResponseMessage::new()
            .add_file(image)
            .add_embed(embed)
            .content(message);

        let response = CreateInteractionResponse::Message(response);
        if let Err(why) = interaction.create_response(&ctx.http, response).await {
            log::warn!("couldn't create interaction: {}", why);
        };

        state::delete(interaction).await;
    } else if game_state.number_of_guesses() > game_state.max_guesses() {
        state::delete(interaction).await;

        let (Some(image), _) = image_fetcher.fetch(game_state.card()).await else {
            log::warn!("couldn't fetch image");
            return;
        };
        let number_of_guesses = game_state.number_of_guesses();
        let guess_plural = if number_of_guesses > 1 {
            "guesses"
        } else {
            "guess"
        };

        let message = MessageBuilder::new()
            .push(format!(
                "You have all failed after {} {}!",
                number_of_guesses, guess_plural
            ))
            .build();

        let embed = game_state.convert_to_embed();

        let response = CreateInteractionResponseMessage::new()
            .add_file(image)
            .add_embed(embed)
            .content(message);

        let response = CreateInteractionResponse::Message(response);
        if let Err(why) = interaction.create_response(&ctx.http, response).await {
            log::warn!("couldn't create interaction: {}", why);
        }
    } else {
        let (Some(illustration), _) = image_fetcher.fetch_illustration(game_state.card()).await
        else {
            log::warn!("couldn't fetch illustration");
            return;
        };
        let remaining_guesses = game_state.max_guesses() - game_state.number_of_guesses() + 1;
        let guess_plural = if remaining_guesses > 1 {
            "guesses"
        } else {
            "guess"
        };

        let response = CreateInteractionResponseMessage::new()
            .content(format!(
                "'{}' was not the correct card. You have {} {} remaining",
                guess, remaining_guesses, guess_plural
            ))
            .add_file(illustration)
            .embed(game_state.to_embed());

        let response = CreateInteractionResponse::Message(response);
        if let Err(why) = interaction.create_response(&ctx.http, response).await {
            log::warn!("couldn't create interaction: {}", why);
        };
        state::add(&game_state, interaction).await;
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("guess")
        .description("Guess the card")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "card",
                "The name of the card you want to guess",
            )
            .required(true),
        )
}

struct Options {
    guess: String,
}

impl ResolveOption for Options {
    fn resolve(options: Vec<(&str, ResolvedValue)>) -> Result<Self, ParseError> {
        let Some((_, guess)) = options.first() else {
            return Err(ParseError::new("Could not get first option"));
        };

        let guess = match guess {
            ResolvedValue::String(guess) => guess.to_string(),
            _ => return Err(ParseError::new("ResolvedValue was not a string")),
        };

        Ok(Options { guess })
    }
}
