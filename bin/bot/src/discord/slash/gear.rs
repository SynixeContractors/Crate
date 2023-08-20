use serenity::{builder::CreateApplicationCommand, model::prelude::{command::CommandOptionType, application_command::{ApplicationCommandInteraction, CommandDataOption}}, prelude::Context};

use crate::{discord::interaction::{Interaction, Generic}, get_option};

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
    return interaction.reply(
        format!("The weapon is {} and the paint is {}",
        weapon,
        paint,
    )
    ).await;
}
