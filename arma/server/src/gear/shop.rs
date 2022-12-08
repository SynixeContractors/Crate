use std::collections::HashMap;

use arma_rs::{Group, IntoArma};
use serenity::model::prelude::UserId;
use synixe_events::gear::db;
use synixe_proc::events_request;

use crate::{CONTEXT, RUNTIME};

pub fn group() -> Group {
    Group::new()
        .command("items", command_items)
        .command("enter", command_enter)
}

fn command_items() {
    RUNTIME.spawn(async {
        let Ok(((db::Response::ShopGetAll(Ok(items)), _), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            ShopGetAll {}
        ).await else {
            error!("failed to fetch shop items over nats");
            CONTEXT.read().await.as_ref().unwrap().callback_null("crate:gear:shop", "items:err");
            return;
        };
        CONTEXT
            .read()
            .await
            .as_ref()
            .unwrap()
            .callback_null("crate:gear:shop", "items:clear");
        for (class, (roles, price)) in items {
            CONTEXT.read().await.as_ref().unwrap().callback_data(
                "crate:gear:shop",
                "items:set",
                vec![
                    class.to_arma(),
                    arma_rs::Value::Array(vec![
                        roles.unwrap_or_default().to_arma(),
                        price.to_arma(),
                    ]),
                ],
            );
        }
        CONTEXT
            .read()
            .await
            .as_ref()
            .unwrap()
            .callback_null("crate:gear:shop", "items:publish");
    });
}

fn command_enter(discord: String, steam: String, items: HashMap<String, i32>) {
    RUNTIME.spawn(async move {
        debug!("entering shop for {} with {:?} items", discord, items.len());
        let Ok(((db::Response::ShopEnter(Ok((locker, balance))), _), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            ShopEnter {
                member: UserId(discord.parse().unwrap()),
                items,
            }
        ).await else {
            error!("failed to enter shop over nats");
            CONTEXT.read().await.as_ref().unwrap().callback_data(
                "crate:gear:shop",
                "enter:err",
                vec![steam],
            );
            return;
        };
        CONTEXT.read().await.as_ref().unwrap().callback_data(
            "crate:gear:shop",
            "enter:ok",
            vec![steam.to_arma(), locker.to_arma(), balance.to_arma()],
        );
        debug!("shop entered for {}", discord);
    });
}
