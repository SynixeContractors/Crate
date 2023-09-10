use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        command::CommandOptionType,
    },
    prelude::Context,
};

use crate::{
    discord::interaction::{Generic, Interaction},
    get_option,
};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("gear")
        .description("Manage your gear")
        .create_option(|option| {
            option
                .name("repaint")
                .description("Repaint a weapon")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("weapon")
                        .description("Select the weapon you want to modify")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("paint")
                        .description("Select the new paint job for the weapon")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
                })
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for gear provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "repaint" => repaint(ctx, command, &subcommand.options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn repaint(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    let Some(weapon) = get_option!(options, "weapon", String) else {
        return interaction.reply("Invalid weapon").await;
    };
    let Some(paint) = get_option!(options, "paint", String) else {
        return interaction.reply("Invalid paint").await;
    };
    interaction
        .reply(format!("The weapon is {weapon} and the paint is {paint}",))
        .await
}
