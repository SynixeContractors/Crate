use serenity::{
    all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::{
        CreateAutocompleteResponse, CreateCommand, CreateCommandOption, CreateInteractionResponse,
    },
    client::Context,
};
use strum::IntoEnumIterator;
use synixe_meta::{
    discord::role::{DOCKER, MISSION_REVIEWER, STAFF},
    docker::Container,
};
use synixe_proc::events_request_30;

use crate::{discord::interaction::Interaction, get_option};

use super::ShouldAsk;

pub fn register() -> CreateCommand {
    CreateCommand::new("docker")
        .description("Interact with the docker containers")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "restart",
                "Restart a container",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "container",
                    "The container to restart",
                )
                .set_autocomplete(true)
                .required(true),
            ),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for docker provided");
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(options) = &subcommand.value {
        match subcommand.name.as_str() {
            "restart" => restart(ctx, command, options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub async fn autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
) -> serenity::Result<()> {
    let Some(subcommand) = autocomplete.data.options.first() else {
        warn!("No subcommand for bank provided");
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(options) = &subcommand.value {
        match subcommand.name.as_str() {
            "restart" => container_autocomplete(ctx, autocomplete, options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn container_autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
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
        .create_response(&ctx.http, {
            let mut f = CreateAutocompleteResponse::default();
            for container in containers {
                f = f.add_string_choice(
                    container.name().unwrap_or_else(|| container.id()),
                    container.key(),
                );
            }
            CreateInteractionResponse::Autocomplete(f)
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}

async fn restart(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
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
