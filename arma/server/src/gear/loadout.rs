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
        debug!("fetching loadout for {}", discord);
        let Ok((db::Response::LoadoutGet(Ok(loadout)), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            LoadoutGet {
                member: UserId(discord.parse().unwrap()),
            }
        ).await else {
            error!("failed to fetch loadout over nats");
            CONTEXT.read().await.as_ref().unwrap().callback_data(
                "crate:gear:loadout",
                "get:err",
                vec![steam],
            );
            return;
        };
        if let Some(loadout) = loadout {
            debug!("found loadout for {}", discord);
            CONTEXT.read().await.as_ref().unwrap().callback_data(
                "crate:gear:loadout",
                "get:set",
                vec![steam, loadout],
            );
        } else {
            debug!("no loadout found for {}", discord);
            CONTEXT.read().await.as_ref().unwrap().callback_data(
                "crate:gear:loadout",
                "get:empty",
                vec![steam],
            );
        }
    });
}

fn command_store(discord: String, steam: String, loadout: String) {
    RUNTIME.spawn(async move {
        let Ok((db::Response::LoadoutStore(Ok(())), _)) = events_request!(
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
            CONTEXT.read().await.as_ref().unwrap().callback_data(
                "crate:gear:loadout",
                "store:err",
                vec![steam],
            );
            return;
        };
        CONTEXT.read().await.as_ref().unwrap().callback_data(
            "crate:gear:loadout",
            "store:ok",
            vec![steam],
        );
    });
}
