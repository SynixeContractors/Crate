use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        autocomplete::AutocompleteInteraction,
        command::CommandOptionType,
    },
    prelude::Context,
};
use synixe_proc::events_request_2;
use uuid::Uuid;

use crate::{
    discord::interaction::{Generic, Interaction},
    get_option,
};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("github")
        .description("GitHub commands")
        .create_option(|option| {
            option
                .name("invite")
                .description("Invite a user to the GitHub organization")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("username")
                        .description("GitHub username")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for github provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "invite" => invite(ctx, command, &subcommand.options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn invite(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);

    Ok(())
}
