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

#[allow(clippy::manual_let_else)] // seems to be a false positive
/// Fetches a user's discord id and roles
fn command_get(steam: String, name: String) {
    if steam == "_SP_PLAYER_" {
        return;
    }
    RUNTIME.spawn(async move {
        let Ok(((db::Response::FromSteam(resp), _), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::discord::db,
            FromSteam {
                steam: steam.clone(),
            }
        ).await else {
            error!("failed to fetch discord id over nats");
            CONTEXT.read().await.as_ref().unwrap().callback_data("crate:discord", "member:get:err", vec![
                arma_rs::Value::String(steam),
            ]);
            return;
        };
        let discord_id = if let Ok(Some(discord_id)) = resp { discord_id } else {
            let Ok(((discord::info::Response::MemberByName(Ok(Some(discord_id))), _), _)) = events_request!(
                bootstrap::NC::get().await,
                synixe_events::discord::info,
                MemberByName {
                    name: name.clone(),
                }
            ).await else {
                error!("failed to check for name match over nats");
                audit(format!("Steam account {steam} failed to link using the name {name}")).await;
                CONTEXT.read().await.as_ref().unwrap().callback_data("crate:discord", "member:get:needs_link", vec![steam.clone()]);
                return;
            };
            let Ok(((db::Response::SaveSteam(Ok(())), _), _)) = events_request!(
                bootstrap::NC::get().await,
                synixe_events::discord::db,
                SaveSteam {
                    steam: steam.clone(),
                    member: discord_id,
                }
            ).await else {
                error!("failed to save discord id over nats");
                CONTEXT.read().await.as_ref().unwrap().callback_data("crate:discord", "member:get:err", vec![
                    arma_rs::Value::String(steam),
                ]);
                return;
            };
            audit(format!("Steam account {steam} is now linked to <@{discord_id}>")).await;
            discord_id.to_string()
        };
        let Ok(((info::Response::MemberRoles(resp), _), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::discord::info,
            MemberRoles {
                user: UserId(discord_id.parse().unwrap()),
            }
        ).await else {
            error!("failed to fetch discord roles over nats");
            CONTEXT.read().await.as_ref().unwrap().callback_data("crate:discord", "member:get:err", vec![
                arma_rs::Value::String(steam),
            ]);
            return;
        };
        let Ok(roles) = resp else {
            error!("failed to fetch discord roles over nats");
            CONTEXT.read().await.as_ref().unwrap().callback_data("crate:discord", "member:get:err", vec![
                arma_rs::Value::String(steam),
            ]);
            return;
        };
        STEAM_CACHE.write().await.insert(discord_id.clone(), steam.clone());
        CONTEXT.read().await.as_ref().unwrap().callback_data("crate:discord", "member:get:ok", vec![
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
    RUNTIME.spawn(async move {
        let Ok(((db::Response::SaveDLC(Ok(())), _), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::discord::db,
            SaveDLC {
                member: UserId(discord.parse().unwrap()),
                dlc,
            }
        ).await else {
            error!("failed to save dlc over nats");
            return;
        };
    });
}
