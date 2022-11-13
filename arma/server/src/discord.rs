use arma_rs::{Group, IntoArma};
use serenity::model::prelude::UserId;
use synixe_events::discord::{db, info};
use synixe_proc::events_request;

use crate::{CONTEXT, RUNTIME};

pub fn group() -> Group {
    Group::new().command("fetch", command_fetch)
}

/// Fetches a user's discord id and roles
fn command_fetch(steam: String) {
    RUNTIME.spawn(async {
        let Ok(((db::Response::FromSteam(resp), _), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::discord::db,
            FromSteam {
                steam_id: steam.clone(),
            }
        ).await else {
            error!("failed to fetch discord id over nats");
            return;
        };
        let Ok(discord_id) = resp else {
            CONTEXT.read().await.as_ref().unwrap().callback_data("crate_server", "needs_link", steam);
            return;
        };
        let Ok(((info::Response::Roles(resp), _), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::discord::info,
            Roles {
                user: UserId(discord_id.parse().unwrap()),
            }
        ).await else {
            error!("failed to fetch discord roles over nats");
            return;
        };
        let Ok(roles) = resp else {
            error!("failed to fetch discord roles over nats");
            return;
        };
        CONTEXT.read().await.as_ref().unwrap().callback_data("crate_server", "fetch", FetchResponse {
            discord_id,
            roles: roles.into_iter().map(|r| r.to_string()).collect(),
        });
    });
}

struct FetchResponse {
    discord_id: String,
    roles: Vec<String>,
}

impl IntoArma for FetchResponse {
    fn to_arma(&self) -> arma_rs::Value {
        arma_rs::Value::Array(vec![
            arma_rs::Value::String(self.discord_id.clone()),
            arma_rs::Value::Array(
                self.roles
                    .clone()
                    .into_iter()
                    .map(arma_rs::Value::String)
                    .collect(),
            ),
        ])
    }
}

#[cfg(test)]
mod tests {}
