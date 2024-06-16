use serenity::{
    all::{
        CommandData, CommandDataOption, CommandDataOptionValue, CommandInteraction,
        CommandOptionType,
    },
    builder::{
        CreateAutocompleteResponse, CreateCommand, CreateCommandOption, CreateInteractionResponse,
    },
    client::Context,
};
use synixe_events::{missions::db::Response, publish};
use synixe_meta::discord::{
    channel::LOG,
    role::{DOCKER, MISSION_REVIEWER, STAFF},
};
use synixe_proc::events_request_2;

use crate::{discord::interaction::Interaction, get_option};

use super::ShouldAsk;

pub fn register() -> CreateCommand {
    CreateCommand::new("missions")
        .description("Manage missions")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "load",
                "Load a mission onto a server",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "mission",
                    "The mission to load",
                )
                .set_autocomplete(true)
                .required(true),
            ),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for missions provided");
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(values) = &subcommand.value {
        match subcommand.name.as_str() {
            "load" => load(ctx, command, values).await?,
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
        warn!("No subcommand for missions provided");
        return Ok(());
    };
    if subcommand.kind() == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "load" => load_autocomplete(ctx, autocomplete).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn load(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    super::requires_roles(
        command.user.id,
        &[MISSION_REVIEWER, STAFF, DOCKER],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Yes(("missions load", options)),
        &mut interaction,
    )
    .await?;
    let Some(mission_id) = get_option!(options, "mission", String) else {
        return interaction
            .reply("Required option not provided: mission")
            .await;
    };
    if mission_id.starts_with('$') {
        return interaction.reply("Mission ID cannot start with `$`").await;
    }
    let Ok(Ok((Response::FetchMissionList(Ok(missions)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionList {
            search: Some(mission_id.to_string()),
        }
    )
    .await
    else {
        error!("failed to fetch mission list");
        return Ok(());
    };
    if missions.len() != 1 {
        return interaction
            .reply(format!(
                "Found {} missions matching `{}`",
                missions.len(),
                mission_id
            ))
            .await;
    }
    if let Err(e) = publish!(
        bootstrap::NC::get().await,
        synixe_events::missions::publish::Publish::ChangeMission {
            id: missions[0].id.clone(),
            mission_type: missions[0].typ,
            reason: format!("Loaded by <@{}>", command.user.id),
        }
    )
    .await
    {
        error!("failed to publish mission change: {}", e);
        return interaction.reply("Failed to publish mission change").await;
    }
    interaction
        .reply(format!("Mission change requested. Check <#{LOG}>"))
        .await?;
    Ok(())
}

async fn load_autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
) -> serenity::Result<()> {
    let Some(focus) = CommandData::autocomplete(&autocomplete.data) else {
        return Ok(());
    };
    if focus.name != "mission" {
        return Ok(());
    }
    let Ok(Ok((Response::FetchMissionList(Ok(mut missions)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionList {
            search: Some(focus.value.to_string())
        }
    )
    .await
    else {
        error!("failed to fetch mission list");
        return Ok(());
    };
    missions.truncate(25);
    if let Err(e) = autocomplete
        .create_response(&ctx.http, {
            let mut f = CreateAutocompleteResponse::default();
            for mission in missions {
                f = f.add_string_choice(&mission.id, &mission.id);
            }
            CreateInteractionResponse::Autocomplete(f)
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}
