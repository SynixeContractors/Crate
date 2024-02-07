use serenity::{
    all::{CommandData, CommandDataOptionValue, CommandInteraction},
    builder::{CreateAutocompleteResponse, CreateInteractionResponse},
    client::Context,
};
use synixe_events::garage::db::Response;
use synixe_proc::events_request_2;
use uuid::Uuid;

use crate::{
    discord::slash::garage::enums::{AssetFilter, Command},
    get_option,
};

pub async fn autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
) -> serenity::Result<()> {
    let Some(focus) = CommandData::autocomplete(&autocomplete.data) else {
        return Ok(());
    };
    let Some(subcommand) = autocomplete.data.options.first() else {
        return Ok(());
    };
    let Some(command) = Command::from_str(subcommand.name.as_str()) else {
        return Ok(());
    };
    let CommandDataOptionValue::SubCommand(options) = &subcommand.value else {
        return Ok(());
    };
    match command {
        Command::PurchaseVehicle => match focus.name {
            "vehicle" => {
                autocomplete_shop(
                    ctx,
                    autocomplete,
                    AssetFilter::Vehicle(Some(focus.value.to_string())),
                )
                .await
            }
            "color" => autocomplete_color(ctx, autocomplete).await,
            _ => Ok(()),
        },
        Command::PurchaseAddon => match focus.name {
            "vehicle" => {
                autocomplete_vehicle(
                    ctx,
                    autocomplete,
                    command,
                    focus.value.to_string(),
                    VehicleValueType::Id,
                )
                .await
            }
            "addon" => {
                autocomplete_shop(
                    ctx,
                    autocomplete,
                    AssetFilter::Addon(Some(focus.value.to_string())),
                )
                .await
            }
            _ => Ok(()),
        },
        Command::Attach | Command::Detach => match focus.name {
            "vehicle" => {
                autocomplete_vehicle(
                    ctx,
                    autocomplete,
                    command,
                    focus.value.to_string(),
                    VehicleValueType::Plate,
                )
                .await
            }
            "addon" => {
                autocomplete_addon(ctx, autocomplete, {
                    let Some(vehicle) = get_option!(&options, "vehicle", String) else {
                        error!("Missing vehicle option");
                        return Ok(());
                    };
                    vehicle.to_string()
                })
                .await
            }
            _ => Ok(()),
        },
        Command::Spawn => match focus.name {
            "vehicle" => {
                autocomplete_vehicle(
                    ctx,
                    autocomplete,
                    command,
                    focus.value.to_string(),
                    VehicleValueType::Plate,
                )
                .await
            }
            _ => Ok(()),
        },
        Command::View => Ok(()),
    }
}

pub enum VehicleValueType {
    Plate,
    Id,
}

async fn autocomplete_vehicle(
    ctx: &Context,
    autocomplete: &CommandInteraction,
    command: Command,
    filter: String,
    value_type: VehicleValueType,
) -> serenity::Result<()> {
    let Ok(Ok((Response::FetchStoredVehicles(Ok(mut vehicles)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchStoredVehicles {
            stored: Some(true),
            plate: Some(filter)
        }
    )
    .await
    else {
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
    vehicles.truncate(25);
    if let Err(e) = autocomplete
        .create_response(&ctx.http, {
            let mut f = CreateAutocompleteResponse::default();
            for vehicle in vehicles {
                f = f.add_string_choice(
                    format!("{} - {}", vehicle.name, vehicle.plate),
                    // vehicle.plate,
                    match value_type {
                        VehicleValueType::Plate => vehicle.plate,
                        VehicleValueType::Id => vehicle.id.to_string(),
                    },
                );
            }
            CreateInteractionResponse::Autocomplete(f)
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}

async fn autocomplete_color(
    ctx: &Context,
    autocomplete: &CommandInteraction,
) -> serenity::Result<()> {
    let Some(subcommand) = autocomplete.data.options.first() else {
        return Ok(());
    };
    let CommandDataOptionValue::SubCommand(options) = &subcommand.value else {
        return Ok(());
    };
    let Some(id) = options
        .iter()
        .find(|o| o.name == "vehicle")
        .and_then(|o| o.value.as_str())
        .map(|v| Uuid::parse_str(v).expect("Invalid UUID"))
    else {
        error!("Missing vehicle option");
        return Ok(());
    };
    let Ok(Ok((Response::FetchVehicleColors(Ok(colors)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchVehicleColors { id }
    )
    .await
    else {
        error!("Failed to fetch colors");
        return Ok(());
    };

    if let Err(e) = autocomplete
        .create_response(&ctx.http, {
            let mut f = CreateAutocompleteResponse::default();
            for color in colors {
                f = f.add_string_choice(color.name.to_string(), color.name.to_string());
            }
            CreateInteractionResponse::Autocomplete(f)
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}

async fn autocomplete_addon(
    ctx: &Context,
    autocomplete: &CommandInteraction,
    plate: String,
) -> serenity::Result<()> {
    debug!("Autocompleting addons for {}", plate);
    let Ok(Ok((Response::FetchStoredAddons(Ok(mut addons)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchStoredAddons { plate }
    )
    .await
    else {
        error!("Failed to fetch addons");
        return Ok(());
    };
    addons.truncate(25);
    if let Err(e) = autocomplete
        .create_response(&ctx.http, {
            let mut f = CreateAutocompleteResponse::default();
            for addon in addons {
                f = f.add_string_choice(addon.name.to_string(), addon.id);
            }
            CreateInteractionResponse::Autocomplete(f)
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}

async fn autocomplete_shop(
    ctx: &Context,
    autocomplete: &CommandInteraction,
    filter: AssetFilter,
) -> serenity::Result<()> {
    let Ok(Ok((Response::FetchShopAssets(Ok(assets)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchShopAssets {
            search: filter.search()
        }
    )
    .await
    else {
        error!("Failed to fetch all shop assests");
        return Ok(());
    };

    let mut assets: Vec<_> = match filter {
        AssetFilter::Vehicle(_) => assets.into_iter().filter(|a| a.base.is_none()).collect(),
        AssetFilter::Addon(_) => {
            let Some(subcommand) = autocomplete.data.options.first() else {
                return Ok(());
            };
            // options
            let CommandDataOptionValue::SubCommand(options) = &subcommand.value else {
                return Ok(());
            };
            let Some(id) = options
                .iter()
                .find(|o| o.name == "vehicle")
                .and_then(|o| o.value.as_str())
                .map(|v| Uuid::parse_str(v).expect("Invalid UUID"))
            else {
                error!("Missing vehicle option");
                return Ok(());
            };
            assets.into_iter().filter(|a| a.base == Some(id)).collect()
        }
    };
    assets.truncate(25);
    if let Err(e) = autocomplete
        .create_response(&ctx.http, {
            let mut f = CreateAutocompleteResponse::default();
            for assets in assets {
                f = f.add_string_choice(
                    format!(
                        "{} - {}",
                        assets.name,
                        bootstrap::format::money(assets.cost, false)
                    ),
                    assets.id,
                );
            }
            CreateInteractionResponse::Autocomplete(f)
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}
