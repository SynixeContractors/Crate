use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
        command::CommandOptionType,
    },
    prelude::Context,
};

use crate::discord::interaction::{Generic, Interaction};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("dlc")
        .description("See DLC ownership")
        .create_option(|option| {
            option
                .name("member")
                .description("See a member's DLC ownership")
                .kind(CommandOptionType::User)
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command));
    match command
        .data
        .options
        .iter()
        .find(|option| option.name == "member")
    {
        Some(option) => {
            let Some(CommandDataOptionValue::User(user, _member)) = option.resolved.as_ref() else {
                interaction.reply("Invalid user").await?;
                return Ok(());
            };
            interaction.reply(&format!("User: {}", user.name)).await?;
        }
        _ => {
            interaction.reply("Fetching all").await?;
        }
    }
    Ok(())
}
