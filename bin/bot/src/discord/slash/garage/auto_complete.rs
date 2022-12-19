use serenity::{model::prelude::autocomplete::AutocompleteInteraction, prelude::Context};

use synixe_events::garage::db::Response;
use synixe_proc::events_request;

use crate::discord::slash::garage::enums::{AssetFilter, Command};

pub async fn autocomplete(ctx: &Context, autocomplete: &AutocompleteInteraction) {
    let subcommand = autocomplete.data.options.first().unwrap();
    let Some(command) = Command::from_str(subcommand.name.as_str()) else {
        return;
    };
    let focus = subcommand.options.iter().find(|o| o.focused);
    let Some(focus) = focus else {
        return;
    };
    let focus_value = focus.value.as_ref().unwrap().as_str().unwrap().to_string();
    match command {
        Command::PurchaseVehicle => {
            autocomplete_shop(ctx, autocomplete, AssetFilter::Vehicle(Some(focus_value))).await;
        }
        Command::PurchaseAddon => {
            match focus.name.as_str() {
                "vehicle" => autocomplete_shop(ctx, autocomplete, AssetFilter::Vehicle(Some(focus_value))).await,
                "addon" => autocomplete_shop(ctx, autocomplete, AssetFilter::Addon(Some(focus_value))).await,
                _ => (),
            }
        }
        Command::Attach | Command::Detach => {
            match focus.name.as_str() {
                "vehicle" => autocomplete_vehicle(ctx, autocomplete, command, focus_value).await,
                "addon" => autocomplete_addon(ctx, autocomplete, {
                    let Some(vehicle) = subcommand.options.iter().find(|o| o.name == "vehicle") else {
                        return;
                    };
                    info!("vehicle: {:?}", vehicle);
                    let Ok(uuid) = vehicle.value.as_ref().unwrap().as_str().unwrap().to_string().parse() else {
                        return;
                    };
                    uuid
                }).await,
                _ => (),
            }
        }
        Command::View => {}
    }
}

async fn autocomplete_vehicle(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    command: Command,
    filter: String,
) {
    let Ok(Ok((Response::FetchStoredVehicles(Ok(mut vehicles)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchStoredVehicles { stored: Some(true), plate: Some(filter) }
    ).await else {
        error!("Failed to fetch vehicles");
        return;
    };

    match command {
        Command::Attach => vehicles.retain(|v| v.addon.is_none()),
        Command::Detach => vehicles.retain(|v| v.addon.is_some()),
        _ => {}
    }

    if vehicles.len() > 25 {
        vehicles.truncate(25);
    }
    if let Err(e) = autocomplete
        .create_autocomplete_response(&ctx.http, |f| {
            for vehicle in vehicles {
                f.add_string_choice(
                    format!("{} - {}", vehicle.name, vehicle.plate),
                    vehicle.plate,
                );
            }
            f
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
}

async fn autocomplete_addon(ctx: &Context, autocomplete: &AutocompleteInteraction, plate: String) {
    let Ok(Ok((Response::FetchStoredAddons(Ok(mut addons)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchStoredAddons { plate }
    ).await else {
        error!("Failed to fetch addons");
        return;
    };

    if addons.len() > 25 {
        addons.truncate(25);
    }
    if let Err(e) = autocomplete
        .create_autocomplete_response(&ctx.http, |f| {
            for addon in addons {
                f.add_string_choice(addon.name.to_string(), addon.id);
            }
            f
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
}

async fn autocomplete_shop(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    filter: AssetFilter,
) {
    let Ok(Ok((Response::FetchShopAssets(Ok(assets)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchShopAssets { search: filter.search() }
    ).await else {
        error!("Failed to fetch all shop assests");
        return;
    };

    let mut assets: Vec<_> = match filter {
        AssetFilter::Vehicle(_) => assets.into_iter().filter(|a| a.base.is_none()).collect(),
        AssetFilter::Addon(_) => assets.into_iter().filter(|a| a.base.is_some()).collect(),
    };

    if assets.len() > 25 {
        assets.truncate(25);
    }
    if let Err(e) = autocomplete
        .create_autocomplete_response(&ctx.http, |f| {
            for assets in assets {
                f.add_string_choice(format!("{} - ${}", assets.name, assets.cost), assets.id);
            }
            f
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
}
