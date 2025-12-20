use std::{collections::HashMap, sync::RwLock};

use arma_rs::{Context, ContextState, Group};
use serenity::all::UserId;
use synixe_events::gear::db::FuelType;
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

type FuelingState = RwLock<HashMap<(String, String), (f64, UserId, String, FuelType)>>;

#[derive(Default)]
pub struct Fueling(FuelingState);
impl Fueling {
    pub const fn as_ref(&self) -> &FuelingState {
        &self.0
    }
}

fn started(
    ctx: Context,
    source: String,
    target: String,
    discord: String,
    plate: String,
    fuel_type: String,
) {
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
    let fuel_type = FuelType::try_from(fuel_type.as_str()).unwrap_or_else(|()| {
        error!("invalid fuel type: {fuel_type}, defaulting to regular");
        FuelType::Regular
    });
    info!("started fueling from {source} to {target} for discord id {discord}");
    fueling.insert(
        (source, target),
        (0.0, UserId::new(discord), plate, fuel_type),
    );
}

fn tick(ctx: Context, source: String, target: String, amount: f64) {
    let fueling = ctx
        .group()
        .get::<Fueling>()
        .expect("Unable to get fueling state");
    let mut fueling = fueling
        .as_ref()
        .write()
        .expect("Unable to lock fueling state");
    let entry =
        fueling
            .entry((source, target))
            .or_insert((0.0, BRODSKY, String::new(), FuelType::Regular));
    info!("tick fueling from {}: +{}", entry.0, amount);
    entry.0 += amount;
}

fn finished(ctx: Context, source: String, target: String, map: String) {
    info!("finished fueling from {source} to {target}");
    let fueling = ctx
        .group()
        .get::<Fueling>()
        .expect("Unable to get fueling state");
    let mut fueling = fueling
        .as_ref()
        .write()
        .expect("Unable to lock fueling state");
    let Some((amount, discord, plate, fuel_type)) =
        fueling.remove(&(source.clone(), target.clone()))
    else {
        error!("finished fueling called without started for {source} -> {target}");
        return;
    };
    RUNTIME.spawn(async move {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let Ok(Ok((synixe_events::gear::db::Response::Fuel(Ok(spent)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            Fuel {
                member: discord,
                amount: amount.round() as u64,
                fuel_type,
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
