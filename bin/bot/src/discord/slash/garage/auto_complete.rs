use serenity::{model::prelude::autocomplete::AutocompleteInteraction, prelude::Context};

use serenity::model::application::interaction::application_command::CommandDataOption;
use synixe_events::garage::db::Response;
use synixe_proc::events_request;

use crate::discord::slash::garage::enums::{GarageCommands, GarageSubCommands};
use super::purchase;

pub async fn autocomplete(ctx: &Context, autocomplete: &AutocompleteInteraction) {
    let subcommand = autocomplete.data.options.first().unwrap();
    match GarageCommands::from_str(subcommand.name.as_str()).unwrap() {
        GarageCommands::PurchaseVehicle | GarageCommands::PurchaseAddon => {
            purchase::purchase_autocomplete(ctx, autocomplete, &subcommand.options).await
        }
        GarageCommands::Attach => attach_autocomplete(ctx, autocomplete, &subcommand.options).await,
        _ => {}
    }
}

async fn attach_autocomplete(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    options: &[CommandDataOption],
) {
    let focus = options.iter().find(|o| o.focused);
    let Some(focus) = focus else {
        return;
    };
    match GarageSubCommands::from_str(focus.name.as_str()).unwrap() {
        GarageSubCommands::Vehicle => autocomplete_vehicle(ctx, autocomplete, &focus).await,
        // "addon" => autocomplete_addon(ctx, autocomplete, &focus).await,
        _ => unreachable!(),
    }
}

async fn autocomplete_vehicle(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    focus: &CommandDataOption,
) {
    let Ok((Response::FetchVehicleAssets(Ok(vehicles)), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchVehicleAssets { stored: None, plate: None }
    ).await else {
        error!("Failed to fetch vehicles");
        return;
    };

    let mut vehicles: Vec<_> = vehicles
        .into_iter()
        .filter(|c| {
            c.plate.to_lowercase().contains(
                &focus
                    .value
                    .as_ref()
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
                    .to_lowercase(),
            )
        })
        .collect();
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
