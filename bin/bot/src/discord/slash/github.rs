use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        command::CommandOptionType,
    },
    prelude::Context,
};
use synixe_events::github::{db::Response, executions};
use synixe_proc::{events_request_2, events_request_5};

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
    let Some(github) = get_option!(options, "username", String) else {
        interaction.reply("No GitHub username provided").await?;
        return Ok(());
    };

    // Check if a user is already linked
    let Ok(Ok((Response::UserByDiscord(Ok(existing)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::github::db,
        UserByDiscord {
            discord: command.user.id,
        }
    )
    .await else {
        return interaction.reply("Failed to fetch balance").await;
    };

    if let Some(existing) = existing {
        interaction
            .reply(format!("You are already linked to GitHub user {existing}",))
            .await?;
        return Ok(());
    }

    // Link the user
    let Ok(Ok((Response::Link(Ok(())), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::github::db,
        Link {
            discord: command.user.id,
            github: github.clone(),
        }
    )
    .await else {
        return interaction.reply("Failed to link GitHub account").await;
    };

    // Invite the user
    let Ok(Ok((executions::Response::Invite(Ok(())), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::github::executions,
        Invite {
            github: github.clone(),
        }
    )
    .await else {
        return interaction.reply("Failed to invite GitHub user").await;
    };

    interaction.confirm(
        &format!("Are you sure you want to invite {github} to the GitHub organization?\nhttps://github.com/{github}")
    ).await?;
    Ok(())
}
