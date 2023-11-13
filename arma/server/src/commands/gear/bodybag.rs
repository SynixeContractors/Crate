use std::collections::HashMap;

use arma_rs::Group;
use serenity::model::prelude::UserId;
use synixe_events::gear::db;
use synixe_proc::events_request_5;

use crate::{CONTEXT, RUNTIME};

use super::clean_items;

pub fn group() -> Group {
    Group::new().command("store", command_store)
}

fn command_store(discord: String, mut items: HashMap<String, i32>, net_id: String) {
    let Ok(discord) = discord.parse::<u64>() else {
        error!("invalid discord id: {}", discord);
        return;
    };
    clean_items(&mut items);
    RUNTIME.spawn(async move {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        debug!("storing bodybag items for {}: {:?}", discord, items);
        let Ok(Ok((db::Response::LockerStore(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            LockerStore {
                member: UserId(discord),
                items,
                reason: "bodybag".to_string(),
            }
        )
        .await
        else {
            error!("failed to store bodybag items over nats");
            if let Err(e) = context.callback_data("crate:gear:bodybag", "store:err", vec![net_id]) {
                error!("error sending bodybag:store:err: {:?}", e);
            }
            return;
        };
        if let Err(e) = context.callback_data("crate:gear:bodybag", "store:ok", vec![net_id]) {
            error!("error sending bodybag:store:ok: {:?}", e);
        }
        debug!("lockerstore for {}", discord);
    });
}
