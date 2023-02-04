use serenity::{model::prelude::autocomplete::AutocompleteInteraction, prelude::Context};

use synixe_events::garage::db::Response;
use synixe_proc::events_request;

use crate::{
    discord::slash::garage::enums::{AssetFilter, Command},
    get_option,
};

pub async fn autocomplete(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
) -> serenity::Result<()> {
    let Some(subcommand) = autocomplete.data.options.first() else {
        return Ok(());
    };
    let Some(command) = Command::from_str(subcommand.name.as_str()) else {
        return Ok(());
    };
    let focus = subcommand.options.iter().find(|o| o.focused);
    let Some(focus) = focus else {
        return Ok(());
    };
    let Some(focus_option) = focus.value.as_ref() else {
        return Ok(());
    };
    let Some(focus_value) = focus_option.as_str() else {
        return Ok(());
    };
    let focus_value = focus_value.to_string();
    match command {
        Command::PurchaseVehicle => {
            autocomplete_shop(ctx, autocomplete, AssetFilter::Vehicle(Some(focus_value))).await
        }
        Command::PurchaseAddon => match focus.name.as_str() {
            "vehicle" => autocomplete_vehicle(ctx, autocomplete, command, focus_value).await,
            "addon" => {
                autocomplete_shop(ctx, autocomplete, AssetFilter::Addon(Some(focus_value))).await
            }
            _ => Ok(()),
        },
        Command::Attach | Command::Detach => {
            match focus.name.as_str() {
                "vehicle" => autocomplete_vehicle(ctx, autocomplete, command, focus_value).await,
                "addon" => autocomplete_addon(ctx, autocomplete, {
                    let Some(vehicle) = get_option!(&subcommand.options, "vehicle", String) else {
                        error!("Missing vehicle option");
                        return Ok(());
                    };
                    vehicle.to_string()
                })
                .await,
                _ => Ok(()),
            }
        }
        Command::View => Ok(()),
    }
}

async fn autocomplete_vehicle(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    command: Command,
    filter: String,
) -> serenity::Result<()> {
    let Ok(Ok((Response::FetchStoredVehicles(Ok(mut vehicles)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchStoredVehicles { stored: Some(true), plate: Some(filter) }
    ).await else {
        error!("Failed to fetch vehicles");
        return Ok(());
    };

    match command {
        Command::Attach => {
            vehicles.retain(|v| v.addon.is_none() && v.addons.unwrap_or_default() > 0);
        }
        Command::Detach => vehicles.retain(|v| v.addon.is_some()),
        Command::PurchaseAddon => vehicles.retain(|v| v.addons.unwrap_or_default() > 0),
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
    Ok(())
}

async fn autocomplete_addon(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    plate: String,
) -> serenity::Result<()> {
    debug!("Autocompleting addons for {}", plate);
    let Ok(Ok((Response::FetchStoredAddons(Ok(mut addons)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchStoredAddons { plate }
    ).await else {
        error!("Failed to fetch addons");
        return Ok(());
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
    Ok(())
}

async fn autocomplete_shop(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    filter: AssetFilter,
) -> serenity::Result<()> {
    let Ok(Ok((Response::FetchShopAssets(Ok(assets)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchShopAssets { search: filter.search() }
    ).await else {
        error!("Failed to fetch all shop assests");
        return Ok(());
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
                f.add_string_choice(
                    format!(
                        "{} - ${}",
                        assets.name,
                        bootstrap::format::money(assets.cost)
                    ),
                    assets.id,
                );
            }
            f
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}
