use serenity::all::{
    ChannelType, CommandOptionType, CreateCommand, CreateCommandOption, Permissions,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("spoilers")
        .description("Manage automatic spoiler announcements for this server")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "subscribe",
                "[Beta] Start posting new-card spoilers to a channel",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "Channel to post spoilers in",
                )
                .channel_types(vec![ChannelType::Text])
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "unsubscribe",
                "Stop posting spoilers to a channel",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "Channel to stop posting spoilers in",
                )
                .channel_types(vec![ChannelType::Text])
                .required(true),
            ),
        )
}
