use arma_rs::Group;
use serenity::model::prelude::UserId;
use synixe_events::discord::{self, db, info};
use synixe_proc::events_request;

use crate::{audit, CONTEXT, RUNTIME, STEAM_CACHE};

pub fn group() -> Group {
    Group::new()
        .command("get", command_get)
        .command("save_dlc", command_save_dlc)
}

/// Fetches a user's discord id and roles
#[allow(clippy::manual_let_else)]
fn command_get(steam: String, name: String) {
    if steam == "_SP_PLAYER_" {
        return;
    }
    RUNTIME.spawn(async move {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        let Ok(Ok((db::Response::FromSteam(resp), _))) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::discord::db,
            FromSteam {
                steam: steam.clone(),
            }
        ).await else {
            error!("failed to fetch discord id over nats");
            context.callback_data("crate:discord", "member:get:err", vec![
                arma_rs::Value::String(steam),
            ]);
            return;
        };
        let discord_id = if let Ok(Some(discord_id)) = resp { discord_id } else {
            let Ok(Ok((discord::info::Response::MemberByName(Ok(Some(discord_id))), _))) = events_request!(
                bootstrap::NC::get().await,
                synixe_events::discord::info,
                MemberByName {
                    name: name.clone(),
                }
            ).await else {
                error!("failed to check for name match over nats");
                audit(format!("Steam account {steam} failed to link using the name {name}")).await;
                context.callback_data("crate:discord", "member:get:needs_link", vec![steam.clone()]);
                return;
            };
            let Ok(Ok((db::Response::SaveSteam(Ok(())), _))) = events_request!(
                bootstrap::NC::get().await,
                synixe_events::discord::db,
                SaveSteam {
                    steam: steam.clone(),
                    member: discord_id,
                }
            ).await else {
                error!("failed to save discord id over nats");
                context.callback_data("crate:discord", "member:get:err", vec![
                    arma_rs::Value::String(steam),
                ]);
                return;
            };
            audit(format!("Steam account {steam} is now linked to <@{discord_id}>")).await;
            discord_id.to_string()
        };
        let Ok(discord_id_u64) = discord_id.parse::<u64>() else {
            error!("failed to parse discord id");
            context.callback_data("crate:discord", "member:get:err", vec![
                arma_rs::Value::String(steam),
            ]);
            return;
        };
        let Ok(Ok((info::Response::MemberRoles(resp), _))) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::discord::info,
            MemberRoles {
                user: UserId(discord_id_u64),
            }
        ).await else {
            error!("failed to fetch discord roles over nats");
            context.callback_data("crate:discord", "member:get:err", vec![
                arma_rs::Value::String(steam),
            ]);
            return;
        };
        let Ok(roles) = resp else {
            error!("failed to fetch discord roles over nats");
            context.callback_data("crate:discord", "member:get:err", vec![
                arma_rs::Value::String(steam),
            ]);
            return;
        };
        STEAM_CACHE.write().await.insert(discord_id.clone(), steam.clone());
        context.callback_data("crate:discord", "member:get:ok", vec![
            arma_rs::Value::String(steam),
            arma_rs::Value::String(discord_id),
            arma_rs::Value::Array(
                roles
                    .into_iter()
                    .map(|r| r.to_string())
                    .map(arma_rs::Value::String)
                    .collect(),
            ),
        ]);
    });
}

fn command_save_dlc(discord: String, dlc: Vec<u32>) {
    let Ok(discord_u64) = discord.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let Ok(Ok((db::Response::SaveDLC(Ok(())), _))) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::discord::db,
            SaveDLC {
                member: UserId(discord_u64),
                dlc,
            }
        ).await else {
            error!("failed to save dlc over nats");
            return;
        };
    });
}
