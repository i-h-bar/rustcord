use crate::app::App;
use crate::card_store::CardStore;
use crate::game::state;
use crate::image_store::ImageStore;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, MessageBuilder,
};
use crate::cache::Cache;

impl<IS, CS, C> App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    pub async fn give_up_command(&self, ctx: &Context, interaction: &CommandInteraction) {
        let Some(game_state) = state::fetch(ctx, interaction, &self.cache).await else {
            return;
        };

        state::delete(interaction, &self.cache).await;

        let Ok(images) = self.image_store.fetch(game_state.card()).await else {
            log::warn!("couldn't fetch image");
            return;
        };

        let image = CreateAttachment::bytes(
            images.front,
            format!("{}.png", game_state.card().front_image_id()),
        );

        let number_of_guesses = game_state.number_of_guesses();
        let guess_plural = if number_of_guesses > 1 {
            "guesses"
        } else {
            "guess"
        };

        let message = MessageBuilder::new()
            .mention(&interaction.user)
            .push(format!(
                " has given up after {number_of_guesses} {guess_plural}!",
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
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("give_up").description("Give up on the current game")
}
