use crate::game::state::GameState;
use crate::mtg::images::ImageFetcher;
use crate::redis::Redis;
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

    let Some(redis) = Redis::instance() else {
        log::warn!("failed to get redis instance");
        return;
    };

    let Some(game_state_string): Option<String> =
        redis.get(interaction.channel_id.to_string()).await
    else {
        let message = MessageBuilder::new()
            .mention(&interaction.user)
            .push(" no game found in ")
            .push(
                interaction
                    .channel_id
                    .name(ctx)
                    .await
                    .unwrap_or(interaction.channel_id.to_string()),
            )
            .build();
        let response = CreateInteractionResponseMessage::new().content(message);

        let response = CreateInteractionResponse::Message(response);
        if let Err(why) = interaction.create_response(&ctx.http, response).await {
            log::warn!("couldn't create interaction: {}", why);
        };

        return;
    };

    let game_state = match ron::from_str::<GameState>(&game_state_string) {
        Ok(mut game_state) => {
            game_state.add_guess();
            game_state
        }
        Err(why) => {
            log::warn!("Couldn't parse game state: {}", why);
            return;
        }
    };

    let Some(image_fetcher) = ImageFetcher::get() else {
        log::warn!("couldn't get image fetcher");
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

        if let Err(why) = redis.delete(interaction.channel_id.to_string()).await {
            log::warn!(
                "Error deleting key: '{}' from redis the response: {:?}",
                interaction.channel_id.to_string(),
                why
            );
        };
    } else if game_state.number_of_guesses() > game_state.max_guesses() {
        if let Err(why) = redis.delete(interaction.channel_id.to_string()).await {
            log::warn!(
                "Error deleting key: '{}' from redis the response: {:?}",
                interaction.channel_id.to_string(),
                why
            );
        }

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

        let response = CreateInteractionResponseMessage::new()
            .content(format!("'{}' was not the correct card", guess))
            .add_file(illustration)
            .embed(game_state.to_embed());

        let response = CreateInteractionResponse::Message(response);
        if let Err(why) = interaction.create_response(&ctx.http, response).await {
            log::warn!("couldn't create interaction: {}", why);
        };

        if let Err(why) = redis
            .set(
                interaction.channel_id.to_string(),
                ron::to_string(&game_state).unwrap(),
            )
            .await
        {
            log::warn!("Error while trying to set value in redis: {}", why);
        };
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
