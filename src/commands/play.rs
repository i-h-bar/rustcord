use crate::db::Psql;
use crate::game::state::{Difficulty, GameState};
use crate::mtg::images::ImageFetcher;
use crate::redis::Redis;
use crate::utils::{fuzzy_match_set_name, parse};
use serenity::all::{
    CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, MessageBuilder, ResolvedValue,
};
use serenity::prelude::*;
use crate::utils;
use crate::utils::parse::{ParseError, ResolveOption};

pub async fn run(ctx: &Context, interaction: &CommandInteraction) {
    let Options { set } = match parse::options(interaction.data.options()) {
        Ok(options) => options,
        Err(err) => {
            log::warn!("{}", err);
            return;
        }
    };
    let Some(db) = Psql::get() else {
        log::warn!("failed to get Psql database");
        return;
    };
    
    let random_card = if let Some(set_name) = set {
        let matched_set = if set_name.chars().count() < 5 {
            utils::set_from_abbreviation(&set_name).await
        } else {
            fuzzy_match_set_name(&utils::normalise(&set_name)).await 
        };
        
        let Some(matched_set) = matched_set else {
            let message = MessageBuilder::new()
                .mention(&interaction.user)
                .push(" could not find set named ")
                .push(set_name)
                .build();

            let response = CreateInteractionResponseMessage::new().content(message);
            if let Err(why) = interaction
                .create_response(ctx, CreateInteractionResponse::Message(response))
                .await
            {
                log::error!("couldn't create interaction response: {:?}", why);
            }
            return;
        };
        db.random_card_from_set(&matched_set).await
    } else {
        db.random_distinct_card().await
    };

    if let Some(card) = random_card {
        let Some(image_fetcher) = ImageFetcher::get() else {
            log::warn!("failed to get image fetcher");
            return;
        };
        let (Some(illustration), _) = image_fetcher.fetch_illustration(&card).await else {
            log::warn!("failed to get image");
            return;
        };

        let game_state = GameState::from(card, Difficulty::Easy);

        let response = CreateInteractionResponseMessage::new()
            .add_file(illustration)
            .add_embed(game_state.to_embed());

        let response = CreateInteractionResponse::Message(response);
        if let Err(why) = interaction.create_response(&ctx.http, response).await {
            log::error!("couldn't create interaction response: {:?}", why);
        };
        let Some(redis) = Redis::instance() else {
            log::warn!("failed to get redis connection");
            return;
        };
        if let Err(why) = redis
            .set(
                interaction.channel_id.to_string(),
                ron::to_string(&game_state).unwrap(),
            )
            .await
        {
            log::warn!("couldn't set redis value: {:?}", why);
        };
    } else {
        log::warn!("Failed to get random card")
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("play")
        .description("Start a guess the card game")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "set",
                "What set to choose the card from",
            )
            .required(false),
        )
}

struct Options{
    set: Option<String>,
}


impl ResolveOption for Options {
    fn resolve(option: Vec<(&str, ResolvedValue)>) -> Result<Self, ParseError> {
        let Some((_, set_option)) = option.first() else { return Ok(Options{ set: None }) };
        
        let set = match set_option {
            ResolvedValue::String(card) => { Some(card.to_string()) }
            _ => { return Err(ParseError::new("ResolvedValue was not a string")) }
        };
        
        Ok(Options { set })
    }
}

