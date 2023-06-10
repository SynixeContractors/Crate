use std::collections::HashMap;

use arma_rs::{Context, ContextState, Group, Value};
use nats::asynk::Message;
use serenity::model::prelude::UserId;
use synixe_events::{
    garage::{arma::Response, db},
    respond,
};
use synixe_proc::events_request_5;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{audit, RUNTIME};

#[derive(Default)]
pub struct PendingSpawn(RwLock<HashMap<Uuid, Message>>);
impl PendingSpawn {
    pub const fn as_ref(&self) -> &RwLock<HashMap<Uuid, Message>> {
        &self.0
    }
}

pub fn group() -> Group {
    Group::new().command("spawn", spawn).command("store", store)
}

fn spawn(ctx: Context, id: Uuid, res: String) {
    RUNTIME.spawn(async move {
        let pending = ctx
            .global()
            .get::<PendingSpawn>()
            .expect("Unable to get pending spawns");
        let pending = pending.as_ref().read().await;
        let msg = pending.get(&id).expect("Unable to get message");
        let res = res
            .parse::<synixe_events::garage::arma::SpawnResult>()
            .expect("Unable to parse bool");
        if let Err(e) = respond!(msg, Response::Spawn(Ok(res))).await {
            error!("Unable to respond to CanSpawn: {}", e);
        }
    });
}

fn store(ctx: Context, plate: String, state: HashMap<String, Value>, discord: String) {
    let Ok(discord) = discord.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let Ok(Ok((db::Response::StoreVehicle(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::garage::db,
            StoreVehicle {
                plate: plate.to_string(),
                member: UserId(discord),
                state: serde_json::Value::Object(
                    state
                        .into_iter()
                        .map(|(k, v)| (k, v.to_json()))
                        .collect()
                ),
            }
        )
        .await
        else {
            error!("failed to store vehicle over nats");
            return;
        };
        audit(format!(
            "
            vehicle {plate} stored by <@{discord}>",
        ))
        .await;
        if let Err(e) = ctx.callback_data("crate:garage", "store", vec![plate]) {
            error!("error sending store: {:?}", e);
        }
    });
}
