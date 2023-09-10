use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        command::CommandOptionType,
    },
    prelude::Context,
};
use synixe_events::{
    discord::write::{DiscordContent, DiscordMessage},
    github::db::Response,
};
use synixe_meta::discord::role::STAFF;
use synixe_proc::{events_request_2, events_request_5};

use crate::{
    discord::interaction::{Generic, Interaction},
    get_option, get_option_user,
};

use super::ShouldAsk;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("github")
        .description("GitHub commands")
        .create_option(|option| {
            option
                .name("link")
                .description("link a member to their GitHub account")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("member")
                        .description("Member to join")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
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
    super::requires_roles(
        command.user.id,
        &[STAFF],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Deny,
        &mut interaction,
    )
    .await?;

    let Some(github) = get_option!(options, "username", String) else {
        interaction.reply("No GitHub username provided").await?;
        return Ok(());
    };

    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };
    if user.bot {
        return interaction.reply("You can't link a bot").await;
    }

    // Check if a user is already linked
    let Ok(Ok((Response::UserByDiscord(Ok(existing)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::github::db,
        UserByDiscord {
            discord: command.user.id,
        }
    )
    .await
    else {
        return interaction.reply("Failed to fetch github user").await;
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
    .await
    else {
        return interaction.reply("Failed to link GitHub account").await;
    };

    interaction
        .reply(format!(
            "Linked GitHub account `{github}` to <@{}>",
            user.id
        ))
        .await?;

    if let Err(e) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::discord::write,
        Audit {
            message: DiscordMessage {
                content: DiscordContent::Text(format!(
                    "<@{}> linked GitHub account `{}` to <@{}>",
                    github, command.user.id, user.id,
                )),
                reactions: vec![],
            }
        }
    )
    .await
    {
        error!("Failed to audit vehicle spawn: {}", e);
    }
    Ok(())
}
