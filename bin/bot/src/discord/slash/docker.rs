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
    docker::ArmaServer,
};
use synixe_proc::events_request_30;

use crate::{discord::interaction::Interaction, get_option};

use super::ShouldAsk;

pub fn register() -> CreateCommand {
    CreateCommand::new("docker")
        .description("Interact with the docker containers")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "restart", "Restart a server")
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "server",
                        "The server to restart",
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
    let Some(name) = get_option!(options, "server", String) else {
        warn!("No server provided");
        return Ok(());
    };
    let name = name.to_lowercase();
    let mut containers = synixe_meta::docker::ArmaServer::iter()
        .map(|server| (server.to_string().to_lowercase(), server))
        .collect::<Vec<(String, ArmaServer)>>();
    containers.retain(|c| c.0.contains(&name));
    containers.truncate(25);
    if let Err(e) = autocomplete
        .create_response(&ctx.http, {
            let mut f = CreateAutocompleteResponse::default();
            for server in containers {
                f = f.add_string_choice(server.0.clone(), server.0);
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
    interaction.reply("Restarting server...").await?;
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
    let server = get_option!(options, "server", String);
    let Some(server) = server else {
        error!("No server provided");
        return Ok(());
    };
    let Ok(server) = ArmaServer::try_from(server.as_str()) else {
        error!("Invalid server provided: {}", server);
        interaction.reply("Invalid server provided").await?;
        return Ok(());
    };
    if let Err(e) = events_request_30!(
        bootstrap::NC::get().await,
        synixe_events::containers::docker,
        Restart {
            server,
            reason: format!("Restarted by <@{}>", command.user.id),
        }
    )
    .await
    {
        error!("Failed to restart server: {}", e);
        interaction.reply("Failed to request restart").await?;
    } else {
        interaction.reply("Restart requested").await?;
    }
    Ok(())
}
