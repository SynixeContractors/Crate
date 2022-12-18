use std::collections::HashMap;

use arma_rs::{Group, IntoArma};
use serenity::model::prelude::UserId;
use synixe_events::gear::db;
use synixe_proc::events_request;

use crate::{CONTEXT, RUNTIME};

use super::clean_items;

pub fn group() -> Group {
    Group::new()
        .command("items", command_items)
        .command("enter", command_enter)
        .command("leave", command_leave)
        .command("purchase", command_purchase)
}

fn command_items() {
    RUNTIME.spawn(async {
        let Ok(Ok((db::Response::ShopGetAll(Ok(items)), _))) = events_request!(
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

fn command_enter(discord: String, steam: String, mut items: HashMap<String, i32>) {
    clean_items(&mut items);
    RUNTIME.spawn(async move {
        debug!("entering shop for {} with {:?} items", discord, items.len());
        let Ok(Ok((db::Response::ShopEnter(Ok((locker, balance))), _))) = events_request!(
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

fn command_leave(discord: String, steam: String, loadout: String, mut items: HashMap<String, i32>) {
    clean_items(&mut items);
    RUNTIME.spawn(async move {
        debug!("leaving shop for {} with {:?} items", discord, items.len());
        let Ok(Ok((db::Response::ShopLeave(Ok(())), _))) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            ShopLeave {
                member: UserId(discord.parse().unwrap()),
                loadout: loadout.replace("\"\"", "\""),
                items,
            }
        ).await else {
            error!("failed to leave shop over nats");
            CONTEXT.read().await.as_ref().unwrap().callback_data(
                "crate:gear:shop",
                "leave:err",
                vec![steam],
            );
            return;
        };
        CONTEXT.read().await.as_ref().unwrap().callback_data(
            "crate:gear:shop",
            "leave:ok",
            vec![steam.to_arma()],
        );
        debug!("shop left for {}", discord);
    });
}

fn command_purchase(discord: String, steam: String, mut items: HashMap<String, i32>) {
    clean_items(&mut items);
    items.retain(|_, v| *v > 0);
    RUNTIME.spawn(async move {
        debug!("purchasing for {}: {:?}", discord, items);
        let Ok(Ok((db::Response::ShopPurchase(Ok((locker, balance))), _))) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            ShopPurchase {
                member: UserId(discord.parse().unwrap()),
                items,
            }
        ).await else {
            error!("failed to purchase items over nats");
            CONTEXT.read().await.as_ref().unwrap().callback_data(
                "crate:gear:shop",
                "purchase:err",
                vec![steam],
            );
            return;
        };
        CONTEXT.read().await.as_ref().unwrap().callback_data(
            "crate:gear:shop",
            "purchase:ok",
            vec![steam.to_arma(), locker.to_arma(), balance.to_arma()],
        );
        debug!("shop purchase for {}", discord);
    });
}
