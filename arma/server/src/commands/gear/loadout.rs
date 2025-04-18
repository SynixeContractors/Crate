use std::sync::Mutex;

use arma_rs::{Context, ContextState, Group};
use serenity::model::prelude::UserId;
use synixe_events::gear::db;
use synixe_proc::events_request_5;
use uuid::Uuid;

use crate::{CONTEXT, RUNTIME};

pub fn group() -> Group {
    Group::new()
        .command("get", command_get)
        .command("store", command_store)
        .command("campaign", command_campaign)
        .command("reset", command_reset)
}

#[derive(Default)]
pub struct Campaign {
    pub id: Mutex<Option<Uuid>>,
}

impl Campaign {
    pub fn get(&self) -> Option<Uuid> {
        *self.id.lock().expect("failed to lock campaign state")
    }
}

fn command_get(ctx: Context, discord: String, steam: String) {
    let Ok(discord) = discord.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        debug!("fetching loadout for {discord}");
        let Ok(Ok((db::Response::LoadoutGet(Ok(loadout)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            LoadoutGet {
                member: UserId::new(discord),
                campaign: ctx.group().get::<Campaign>().and_then(Campaign::get),
            }
        )
        .await
        else {
            error!("failed to fetch loadout over nats");
            if let Err(e) = context.callback_data("crate:gear:loadout", "get:err", vec![steam]) {
                error!("error sending loadout:get:err: {e:?}");
            }
            return;
        };
        if let Some(loadout) = loadout {
            debug!("found loadout for {discord}");
            if let Err(e) =
                context.callback_data("crate:gear:loadout", "get:set", vec![steam, loadout])
            {
                error!("error sending loadout:get:set: {e:?}");
            }
        } else {
            debug!("no loadout found for {discord}");
            if let Err(e) = context.callback_data("crate:gear:loadout", "get:empty", vec![steam]) {
                error!("error sending loadout:get:empty: {e:?}");
            }
        }
    });
}

fn command_store(ctx: Context, discord: String, steam: String, loadout: String) {
    let Ok(discord) = discord.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        let Ok(Ok((db::Response::LoadoutStore(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            LoadoutStore {
                member: UserId::new(discord),
                loadout, //.replace("\"\"", "\""),
                campaign: ctx.group().get::<Campaign>().and_then(Campaign::get),
            }
        )
        .await
        else {
            error!("failed to save loadout over nats");
            if let Err(e) = context.callback_data("crate:gear:loadout", "store:err", vec![steam]) {
                error!("error sending loadout:store:err: {e:?}");
            }
            return;
        };
        if let Err(e) = context.callback_data("crate:gear:loadout", "store:ok", vec![steam]) {
            error!("error sending loadout:store:ok: {e:?}");
        }
    });
}

fn command_campaign(ctx: Context, campaign: Uuid) -> bool {
    let state = ctx.group().get::<Campaign>().unwrap_or_else(|| {
        ctx.group().set::<Campaign>(Campaign::default());
        ctx.group()
            .get::<Campaign>()
            .expect("failed to get campaign state")
    });
    let mut state = state.id.lock().expect("failed to lock campaign state");
    *state = Some(campaign);
    debug!("set campaign to {campaign}");
    false
}

fn command_reset(ctx: Context) -> bool {
    let state = ctx.group().get::<Campaign>().unwrap_or_else(|| {
        ctx.group().set::<Campaign>(Campaign::default());
        ctx.group()
            .get::<Campaign>()
            .expect("failed to get campaign state")
    });
    let mut state = state.id.lock().expect("failed to lock campaign state");
    *state = None;
    debug!("reset campaign to None");
    false
}
