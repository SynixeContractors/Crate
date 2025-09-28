use serenity::{
    all::{CommandDataOption, CommandInteraction},
    prelude::Context,
};
use synixe_events::garage::{
    arma::{Response, SpawnResult},
    db,
};
use synixe_meta::discord::{channel::LOG, role::LEADERSHIP};
use synixe_proc::{events_request_2, events_request_5};

use crate::{
    discord::{interaction::Interaction, slash::ShouldAsk, utils::audit},
    get_option,
};

pub static SPAWN_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::too_many_lines)]
pub async fn spawn(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    super::super::requires_roles(
        command.user.id,
        &[LEADERSHIP],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Yes(("garage spawn", options)),
        &mut interaction,
    )
    .await?;

    let plate = get_option!(options, "vehicle", String);
    let Some(plate) = plate else {
        return interaction
            .reply("Required option not provided: vehicle")
            .await;
    };
    interaction.reply("Waiting for lock").await?;
    let _lock = SPAWN_LOCK.lock().await;
    let Ok(Ok((db::Response::FetchVehicleInfo(Ok(info)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchVehicleInfo {
            plate: plate.to_string(),
        }
    )
    .await
    else {
        error!("Failed to fetch spawn info");
        return Ok(());
    };
    let Some(info) = info else {
        return interaction.reply("Vehicle not found").await;
    };
    let Some(class) = info.class else {
        return interaction.reply("Vehicle class not found").await;
    };
    let Some(state) = info.state else {
        return interaction.reply("Vehicle state not found").await;
    };
    let Ok(Ok((Response::Spawn(Ok(response)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::garage::arma,
        Spawn {
            class,
            state,
            plate: plate.to_string(),
        }
    )
    .await
    else {
        error!("Failed to spawn vehicle");
        interaction.reply("Failed to spawn vehicle").await?;
        return Ok(());
    };
    match response {
        SpawnResult::Yes => {
            std::mem::drop(interaction.reply("Vehicle spawning").await);
            if let Err(e) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::garage::db,
                RetrieveVehicle {
                    plate: plate.to_string(),
                    member: command.user.id,
                }
            )
            .await
            {
                error!("Failed to retrieve vehicle: {}", e);
                if let Err(e) = LOG
                    .say(
                        &ctx.http,
                        format!("Failed to store retrieve action on vehicle {plate}: {e}"),
                    )
                    .await
                {
                    error!("Failed to send log message: {}", e);
                }
            }
            interaction.reply("Vehicle spawned").await?;
            audit(format!(
                "Vehicle `{}` spawned by <@{}>",
                plate, command.user.id,
            ));
            Ok(())
        }
        SpawnResult::AreaBlocked => interaction.reply("Area blocked").await,
        SpawnResult::NoPlayers => interaction.reply("No players online").await,
        SpawnResult::NoSpawnArea => {
            interaction
                .reply("No spawn area for this type of vehicle")
                .await
        }
        SpawnResult::NoPlayersNear => interaction.reply("Unable to spawn vehicle").await,
    }
}
