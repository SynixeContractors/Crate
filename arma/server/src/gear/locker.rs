use arma_rs::{Group, IntoArma};
use serenity::model::prelude::UserId;
use synixe_events::gear::db;
use synixe_proc::events_request;

use crate::{CONTEXT, RUNTIME};

pub fn group() -> Group {
    Group::new().command("get", command_get)
}

fn command_get(discord: String, steam: String) {
    RUNTIME.spawn(async move {
        let Ok(((db::Response::LockerGet(Ok(items)), _), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            LockerGet {
                member: UserId(discord.parse().unwrap()),
            }
        ).await else {
            error!("failed to fetch locker items over nats");
            return;
        };
        CONTEXT.read().await.as_ref().unwrap().callback_data(
            "crate:gear:locker",
            "get:clear",
            vec![steam.to_string()],
        );
        for (class, count) in items {
            CONTEXT.read().await.as_ref().unwrap().callback_data(
                "crate:gear:locker",
                "get:add",
                vec![steam.to_arma(), class.to_arma(), count.to_arma()],
            );
        }
        CONTEXT.read().await.as_ref().unwrap().callback_data(
            "crate:gear:locker",
            "get:done",
            vec![steam.to_string()],
        );
    });
}
