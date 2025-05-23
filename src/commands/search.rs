use serenity::all::{CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption};
use crate::mtg::db::QueryParams;
use crate::utils::parse;

pub async fn run(ctx: &Context, interaction: &CommandInteraction) {
    let query_params = match parse::options::<QueryParams>(interaction.data.options()) {
        Ok(params) => params,
        Err(err) => {
            log::warn!("{}", err);
            return;
        }
    };
    
    
}


pub fn register() -> CreateCommand {
    CreateCommand::new("search")
        .description("Search for a card")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "name",
                "Name of the card",
            )
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "set",
                "Constrain search to a set",
            )
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "artist",
                "Constrain search to an artist",
            )
            .required(false),
        )
}
