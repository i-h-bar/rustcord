use crate::app::App;
use crate::cache::Cache;
use crate::card_store::CardStore;
use crate::clients::{MessageInteraction, MessageInterationError};
use crate::image_store::{ImageStore, Images};
use crate::mtg::card::FuzzyFound;
use crate::utils::help::HELP;
use crate::{commands, mtg, utils};
use async_trait::async_trait;
use regex::Captures;
use serenity::all::{Command, Context, CreateAttachment, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, EventHandler, Interaction, Message, Ready};
use crate::utils::emoji::add_emoji;
use crate::utils::{italicise_reminder_text, REGEX_COLLECTION};
use crate::utils::colours::get_colour_identity;

#[async_trait]
impl<IS, CS, C> EventHandler for App<IS, CS, C>
where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync,
{
    async fn message(&self, ctx: Context, msg: Message) {
        let interaction = DiscordMessageInteration { ctx, msg };
        
        if interaction.msg.author.id == interaction.ctx.cache.current_user().id || interaction.msg.author.bot {
            return;
        } else if interaction.msg.content == "!help" {
            if let Err(why) = interaction.reply(HELP.to_string()).await {
                log::error!("Error sending help message: {:?}", why);
            };
        } else {
            for card in self.parse_message(&interaction.msg.content).await {
                mtg::card_response(card, &interaction).await;
            }
        }
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        if let Err(err) = Command::create_global_command(&ctx, commands::play::register()).await {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created play command");
        }

        if let Err(err) = Command::create_global_command(&ctx, commands::guess::register()).await {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created guess command");
        }

        if let Err(err) = Command::create_global_command(&ctx, commands::help::register()).await {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created help command");
        }

        if let Err(err) = Command::create_global_command(&ctx, commands::search::register()).await {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created search command");
        }

        if let Err(err) = Command::create_global_command(&ctx, commands::give_up::register()).await
        {
            log::warn!("Could not create command {:?}", err);
        } else {
            log::info!("Created give_up command");
        }

        log::info!("Bot ready!");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            if command.user.bot {
                return;
            }

            log::info!(
                "Received command: {:?} from {}",
                command.data.name,
                command.channel_id
            );

            match command.data.name.as_str() {
                "help" => commands::help::run(&ctx, &command).await,
                "search" => self.search_command(&ctx, &command).await,
                "play" => self.play_command(&ctx, &command).await,
                "guess" => self.guess_command(&ctx, &command).await,
                "give_up" => self.give_up_command(&ctx, &command).await,
                _ => (),
            }
        }
    }
}

struct DiscordMessageInteration {
    ctx: Context,
    msg: Message,
}

impl DiscordMessageInteration {
    fn create_embed(&self, card: FuzzyFound) -> (CreateEmbed, Option<CreateEmbed>) {
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
    
    async fn send_message(&self, message: CreateMessage) -> Result<(), MessageInterationError> {
        match self.msg.channel_id.send_message(&self.ctx.http, message).await {
            Err(why) => {
                Err(MessageInterationError(why.to_string()))
            }
            Ok(response) => {
                log::info!("Sent message to {:?}", response.channel_id);
                Ok(())
            }
        }
    }
}

#[async_trait]
impl MessageInteraction for DiscordMessageInteration {
    async fn send_card(
        &self,
        card: FuzzyFound,
        images: Images,
    ) -> Result<(), MessageInterationError> {
        let front_image =
            CreateAttachment::bytes(images.front, format!("{}.png", card.front_image_id()));
        let back_image = if let Some(back_image) = images.back {
            card.back_image_id().map(|back_image_id| {
                CreateAttachment::bytes(back_image, format!("{back_image_id}.png"))
            })
        } else {
            None
        };

        let (front, back) = self.create_embed(card);
        let message = CreateMessage::new().add_file(front_image).add_embed(front);
        
        self.send_message(message).await?;

        if let Some(back) = back {
            if let Some(back_image) = back_image {
                let message = CreateMessage::new().add_file(back_image).add_embed(back);
                self.send_message(message).await?;
            }
        }

        Ok(())
    }

    async fn reply(&self, message: String) -> Result<(), MessageInterationError> {
        self.msg.channel_id.say(
            &self.ctx, message,
        ).await.map_err(|_| MessageInterationError(String::from("Failed to send message")))?;

        Ok(())
    }
}
