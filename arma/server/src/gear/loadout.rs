use arma_rs::Group;
use serenity::model::prelude::UserId;
use synixe_events::gear::db;
use synixe_proc::events_request;

use crate::{CONTEXT, RUNTIME};

pub fn group() -> Group {
    Group::new()
        .command("get", command_get)
        .command("store", command_store)
}

fn command_get(discord: String, steam: String) {
    RUNTIME.spawn(async move {
        let Ok(((db::Response::LoadoutGet(Ok(loadout)), _), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            LoadoutGet {
                member: UserId(discord.parse().unwrap()),
            }
        ).await else {
            error!("failed to fetch loadout over nats");
            return;
        };
        CONTEXT.read().await.as_ref().unwrap().callback_data(
            "crate:gear",
            "loadout:get",
            vec![steam, loadout],
        );
    });
}

fn command_store(discord: String, loadout: String) {
    RUNTIME.spawn(async move {
        let Ok(((db::Response::LoadoutStore(Ok(())), _), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            LoadoutStore {
                member: UserId(discord.parse().unwrap()),
                loadout: loadout.replace("\"\"", "\""),
            }
        )
        .await
        else {
            error!("failed to save loadout over nats");
            return;
        };
    });
}
