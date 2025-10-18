use std::collections::HashMap;

use arma_rs::{Group, IntoArma};
use serenity::model::prelude::UserId;
use synixe_events::gear::db;
use synixe_proc::events_request_5;

use crate::{CONTEXT, RUNTIME};

use super::clean_items;

pub fn group() -> Group {
    Group::new()
        .command("items", command_items)
        .command("enter", command_enter)
        .command("leave", command_leave)
        .command("purchase", command_purchase)
        .command("pretty", command_pretty)
}

fn command_items() {
    RUNTIME.spawn(async {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        let mut items = HashMap::new();
        let mut page = 0;
        loop {
            let Ok(Ok((db::Response::ShopGetAll(Ok(to_add)), _))) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                ShopGetAll { page }
            )
            .await
            else {
                error!("failed to fetch shop items over nats");
                if let Err(e) = context.callback_null("crate:gear:shop", "items:err") {
                    error!("error sending shop:items:err: {e:?}");
                }
                return;
            };
            items.extend(to_add.0);
            if !to_add.1 {
                break;
            }
            page += 1;
        }

        if let Err(e) = context.callback_null("crate:gear:shop", "items:clear") {
            error!("error sending shop:items:clear: {e:?}");
        }
        for (class, (pretty, roles, price)) in items {
            if let Err(e) = context.callback_data(
                "crate:gear:shop",
                "items:set",
                vec![
                    class.to_arma(),
                    arma_rs::Value::Array(vec![
                        roles.unwrap_or_default().to_arma(),
                        price.to_arma(),
                    ]),
                    pretty.to_arma(),
                ],
            ) {
                error!("error sending shop:items:set: {e:?}");
            }
        }
        if let Err(e) = context.callback_null("crate:gear:shop", "items:publish") {
            error!("error sending shop:items:publish: {e:?}");
        }
    });
}

fn command_enter(discord: String, steam: String, mut items: HashMap<String, i32>) {
    let Ok(discord) = discord.parse::<u64>() else {
        error!("invalid discord id: {discord}");
        return;
    };
    clean_items(&mut items);
    RUNTIME.spawn(async move {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        debug!("entering shop for {} with {:?} items", discord, items.len());
        let Ok(Ok((db::Response::ShopEnter(Ok((locker, balance))), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            ShopEnter {
                member: UserId::new(discord),
                items,
            }
        )
        .await
        else {
            error!("failed to enter shop over nats");
            if let Err(e) = context.callback_data("crate:gear:shop", "enter:err", vec![steam]) {
                error!("error sending shop:enter:err: {e:?}");
            }
            return;
        };
        if let Err(e) = context.callback_data(
            "crate:gear:shop",
            "enter:ok",
            vec![steam.to_arma(), locker.to_arma(), balance.to_arma()],
        ) {
            error!("error sending shop:enter:ok: {e:?}");
        }
        debug!("shop entered for {discord}");
    });
}

fn command_leave(discord: String, steam: String, loadout: String, mut items: HashMap<String, i32>) {
    let Ok(discord) = discord.parse::<u64>() else {
        error!("invalid discord id: {discord}");
        return;
    };
    clean_items(&mut items);
    RUNTIME.spawn(async move {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        debug!("leaving shop for {} with {:?} items", discord, items.len());
        let Ok(Ok((db::Response::ShopLeave(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            ShopLeave {
                member: UserId::new(discord),
                loadout, // .replace("\"\"", "\""),
                items,
            }
        )
        .await
        else {
            error!("failed to leave shop over nats");
            if let Err(e) = context.callback_data("crate:gear:shop", "leave:err", vec![steam]) {
                error!("error sending shop:leave:err: {e:?}");
            }
            return;
        };
        if let Err(e) = context.callback_data("crate:gear:shop", "leave:ok", vec![steam.to_arma()])
        {
            error!("error sending shop:leave:ok: {e:?}");
        }
        debug!("shop left for {discord}");
    });
}

fn command_purchase(discord: String, steam: String, mut items: HashMap<String, i32>) {
    let Ok(discord) = discord.parse::<u64>() else {
        error!("invalid discord id: {discord}");
        return;
    };
    clean_items(&mut items);
    RUNTIME.spawn(async move {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        debug!("purchasing for {discord}: {items:?}");
        let Ok(Ok((db::Response::ShopPurchase(Ok((locker, balance))), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            ShopPurchase {
                member: UserId::new(discord),
                items,
            }
        )
        .await
        else {
            error!("failed to purchase items over nats");
            if let Err(e) = context.callback_data("crate:gear:shop", "purchase:err", vec![steam]) {
                error!("error sending shop:purchase:err: {e:?}");
            }
            return;
        };
        if let Err(e) = context.callback_data(
            "crate:gear:shop",
            "purchase:ok",
            vec![steam.to_arma(), locker.to_arma(), balance.to_arma()],
        ) {
            error!("error sending shop:purchase:ok: {e:?}");
        }
        debug!("shop purchase for {discord}");
    });
}

fn command_pretty(item: String, pretty: String) {
    if item.is_empty() || pretty.is_empty() {
        return;
    }
    RUNTIME.spawn(async move {
        let Ok(Ok((db::Response::SetPrettyName(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            SetPrettyName { item, pretty }
        )
        .await
        else {
            error!("failed to set pretty name over nats");
            return;
        };
    });
}
