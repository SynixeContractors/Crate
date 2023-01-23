use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        autocomplete::AutocompleteInteraction,
        command::CommandOptionType,
    },
    prelude::Context,
};
use synixe_events::{missions::db::Response, publish};
use synixe_meta::discord::{
    channel::LOG,
    role::{MISSION_REVIEWER, STAFF, DOCKER},
};
use synixe_proc::events_request;

use crate::{
    discord::interaction::{Generic, Interaction},
    get_option,
};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("missions")
        .description("Manage missions")
        .create_option(|option| {
            option
                .name("load")
                .description("Load a mission onto a server")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("mission")
                        .description("The mission to load")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
                })
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for missions provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "load" => load(ctx, command, &subcommand.options).await?,
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
        warn!("No subcommand for missions provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand && subcommand.name.as_str() == "load" {
        load_autocomplete(ctx, autocomplete, &subcommand.options).await?;
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn load(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    super::requires_roles(
        &[MISSION_REVIEWER, STAFF, DOCKER],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        &mut interaction,
    )
    .await?;
    let Some(mission_id) = get_option!(options, "mission", String) else {
        return interaction
            .reply("Required option not provided: mission")
            .await;
    };
    let Ok(Ok((Response::FetchMissionList(Ok(missions)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionList {
            search: Some(mission_id.to_string()),
        }
    )
    .await else {
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
    autocomplete: &AutocompleteInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let Some(focus) = options.iter().find(|o| o.focused) else {
        return Ok(());
    };
    if focus.name != "mission" {
        return Ok(());
    }
    let Ok(Ok((Response::FetchMissionList(Ok(mut missions)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionList {
            search: Some(focus.value.as_ref().expect("value should always exist").as_str().expect("discord should enforce string type").to_string())
        }
    )
    .await else {
        error!("failed to fetch mission list");
        return Ok(());
    };
    if missions.len() > 25 {
        missions.truncate(25);
    }
    if let Err(e) = autocomplete
        .create_autocomplete_response(&ctx.http, |f| {
            for mission in missions {
                f.add_string_choice(&mission.id, &mission.id);
            }
            f
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}
