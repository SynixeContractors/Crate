use std::{collections::HashMap, sync::RwLock};

use arma_rs::{Context, ContextState, Group};
use serenity::all::UserId;
use synixe_meta::discord::BRODSKY;
use synixe_proc::events_request_5;

use crate::RUNTIME;

pub fn group() -> Group {
    Group::new()
        .command("started", started)
        .command("tick", tick)
        .command("finished", finished)
        .command("price", price)
        .state(Fueling::default())
}

type FuelingState = RwLock<HashMap<(String, String), (u64, UserId, String)>>;

#[derive(Default)]
pub struct Fueling(FuelingState);
impl Fueling {
    pub const fn as_ref(&self) -> &FuelingState {
        &self.0
    }
}

fn started(ctx: Context, source: String, target: String, discord: String, plate: String) {
    let Ok(discord) = discord.parse::<u64>() else {
        error!("invalid discord id: {discord}");
        return;
    };
    let fueling = ctx
        .group()
        .get::<Fueling>()
        .expect("Unable to get fueling state");
    let mut fueling = fueling
        .as_ref()
        .write()
        .expect("Unable to lock fueling state");
    fueling.insert((source, target), (0, UserId::new(discord), plate));
}

fn tick(ctx: Context, source: String, target: String, amount: u64) {
    let fueling = ctx
        .group()
        .get::<Fueling>()
        .expect("Unable to get fueling state");
    let mut fueling = fueling
        .as_ref()
        .write()
        .expect("Unable to lock fueling state");
    let entry = fueling
        .entry((source, target))
        .or_insert((0, BRODSKY, String::new()));
    entry.0 += amount;
}

fn finished(ctx: Context, source: String, target: String, map: String) {
    let fueling = ctx
        .group()
        .get::<Fueling>()
        .expect("Unable to get fueling state");
    let mut fueling = fueling
        .as_ref()
        .write()
        .expect("Unable to lock fueling state");
    let Some((amount, discord, plate)) = fueling.remove(&(source.clone(), target.clone())) else {
        error!("finished fueling called without started for {source} -> {target}");
        return;
    };
    RUNTIME.spawn(async move {
        let Ok(Ok((synixe_events::gear::db::Response::Fuel(Ok(spent)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            Fuel {
                member: discord,
                amount,
                plate: if plate.is_empty() { None } else { Some(plate) },
                map,
            }
        )
        .await
        else {
            error!("failed to process fueling purchase for {discord} of {amount} units");
            return;
        };
        info!("processed fueling purchase for {discord} of {amount} units, spent ${spent}");
    });
}

fn price(ctx: Context, map: String) {
    RUNTIME.spawn(async move {
        let fuel_price =
            if let Ok(Ok((synixe_events::gear::db::Response::FuelPrice(Ok(fuel_price)), _))) =
                events_request_5!(
                    bootstrap::NC::get().await,
                    synixe_events::gear::db,
                    FuelPrice { map }
                )
                .await
            {
                fuel_price
            } else {
                error!("failed to get fuel price for map");
                2.00
            };
        if let Err(e) = ctx.callback_data("crate:fuel", "price", fuel_price) {
            error!("error sending fuel price: {e:?}");
        }
    });
}
