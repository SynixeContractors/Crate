use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        autocomplete::AutocompleteInteraction,
        command::CommandOptionType,
    },
    prelude::Context,
};
use strum::IntoEnumIterator;
use synixe_meta::{
    discord::role::{DOCKER, MISSION_REVIEWER, STAFF},
    docker::Container,
};
use synixe_proc::events_request_30;

use crate::{
    discord::interaction::{Generic, Interaction},
    get_option,
};

use super::ShouldAsk;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("docker")
        .description("Interact with the docker containers")
        .create_option(|option| {
            option
                .name("restart")
                .description("Restart a container")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("container")
                        .description("The container to restart")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
                })
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for docker provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "restart" => restart(ctx, command, &subcommand.options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub async fn autocomplete(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
) -> serenity::Result<()> {
    let Some(subcommand) = autocomplete.data.options.first() else {
        warn!("No subcommand for bank provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand && subcommand.name.as_str() == "restart" {
        container_autocomplete(ctx, autocomplete, &subcommand.options).await?;
    }
    Ok(())
}

async fn container_autocomplete(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let Some(name) = get_option!(options, "container", String) else {
        warn!("No container provided");
        return Ok(());
    };
    let name = name.to_lowercase();
    let mut containers = synixe_meta::docker::Primary::iter()
        .map(std::convert::Into::into)
        .collect::<Vec<Container>>();
    // containers.append(&mut synixe_meta::docker::Reynold::iter().map(|c| c.into()).collect::<Vec<Container>>());
    containers.retain(|c| {
        c.name()
            .unwrap_or_else(|| c.id())
            .to_lowercase()
            .contains(&name)
            || c.id().to_lowercase().contains(&name)
    });
    if containers.len() > 25 {
        containers.truncate(25);
    }
    if let Err(e) = autocomplete
        .create_autocomplete_response(&ctx.http, |f| {
            for container in containers {
                f.add_string_choice(
                    container.name().unwrap_or_else(|| container.id()),
                    container.key(),
                );
            }
            f
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}

async fn restart(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    interaction.reply("Restarting container...").await?;
    super::requires_roles(
        command.user.id,
        &[MISSION_REVIEWER, STAFF, DOCKER],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Yes(("docker restart", options)),
        &mut interaction,
    )
    .await?;
    let container = get_option!(options, "container", String);
    let Some(container) = container else {
        error!("No container provided");
        return Ok(());
    };
    let (dc, id) = container.split_once(':').expect("Invalid container");
    if let Err(e) = events_request_30!(
        bootstrap::NC::get().await,
        synixe_events::containers::docker,
        Restart {
            container: Container::new(id.to_string(), dc.to_string(), None,),
            reason: format!("Restarted by <@{}>", command.user.id),
        }
    )
    .await
    {
        error!("Failed to restart container: {}", e);
        interaction.reply("Failed to request restart").await?;
    } else {
        interaction.reply("Restart requested").await?;
    }
    Ok(())
}
