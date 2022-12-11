use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, autocomplete::AutocompleteInteraction,
        command::CommandOptionType,
    },
    prelude::Context,
};

use serenity::model::application::interaction::application_command::CommandDataOption;
use synixe_events::garage::db::Response;
use synixe_proc::events_request;

use crate::discord::{
    self,
    interaction::{Generic, Interaction},
};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("garage")
        .description("Interact with the garage")
        .create_option(|option| {
            option
                .name("view")
                .description("View the garage inventory")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("purchase")
                .description("purchase an asset")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("asset")
                        .description("The asset to purchase")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("attach")
                .description("Attach an asset to a vehicle")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("vehicle")
                        .description("The vehicle in question")
                        .kind(CommandOptionType::String)
                        .set_autocomplete(true)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("addon")
                        .description("The addon to attach")
                        .kind(CommandOptionType::String)
                        .set_autocomplete(true)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("detach")
                .description("Detach an asset from vehicle")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("vehicle")
                        .description("The vehicle in question")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let subcommand = command.data.options.first().unwrap();
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "view" => view(ctx, command, &subcommand.options).await,
            // "purchase" => purchase(ctx, command, &subcommand.options).await,
            "attach" => attach(ctx, command, &subcommand.options).await,
            // "detach" => detach(ctx, command, &subcommand.options).await,
            _ => unreachable!(),
        }
    }
}

pub async fn autocomplete(ctx: &Context, autocomplete: &AutocompleteInteraction) {
    let subcommand = autocomplete.data.options.first().unwrap();
    if subcommand.kind == CommandOptionType::SubCommand && subcommand.name.as_str() == "attach" {
        attach_autocomplete(ctx, autocomplete, &subcommand.options).await;
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
    match focus.name.as_str() {
        "vehicle" => autocomplete_vehicle(ctx, autocomplete, &focus).await,
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

// async fn autocomplete_vehicle(ctx: &Context, autocomplete: &AutocompleteInteraction, focus: &CommandDataOption) {
//     let Ok((Response::FetchVehicleAssets(Ok(vehicles)), _)) = events_request!(
//         bootstrap::NC::get().await,
//         synixe_events::garage::db,
//         FetchVehicleAssets{ stored: None}
//     ).await else {
//         error!("Failed to fetch vehicles");
//         return;
//     };

//     let mut vehicles: Vec<_> = vehicles
//         .into_iter()
//         .filter(|c| {
//             c.plate.to_lowercase().contains(
//                 &focus
//                     .value
//                     .as_ref()
//                     .unwrap()
//                     .as_str()
//                     .unwrap()
//                     .to_string()
//                     .to_lowercase(),
//             )
//         })
//         .collect();
//     if vehicles.len() > 25 {
//         vehicles.truncate(25);
//     }
//     if let Err(e) = autocomplete
//         .create_autocomplete_response(&ctx.http, |f| {
//             for vehicle in vehicles {
//                 f.add_string_choice(&vehicle.name, vehicle.plate);
//             }
//             f
//         })
//         .await
//     {
//         error!("failed to create autocomplete response: {}", e);
//     }
// }

async fn view(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _options: &[CommandDataOption],
) {
    let mut interaction =
        Interaction::new(ctx, discord::interaction::Generic::Application(command));
    interaction.reply("This is the garage view command").await;

    let Ok((Response::FetchVehicleAssets(Ok(vehicles)), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchVehicleAssets{ stored: None, plate: None }
    ).await else {
        interaction.reply("Error fetching vehicle assests").await;
        return;
    };

    if vehicles.is_empty() {
        interaction.reply("No vehicle assests found").await;
        return;
    }

    let mut content = format!("**Vehicle Assests**\n\n");
    for vehicle in vehicles {
        content.push_str(&format!(
            "**{} - stored: {}**\n",
            vehicle.plate, vehicle.stored
        ));
    }
    interaction.reply(content).await;
}

async fn attach(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) {
    let mut interaction =
        Interaction::new(ctx, discord::interaction::Generic::Application(command));
    interaction.reply("This is the garage attach command").await;

    let plate = options
        .iter()
        .find(|option| option.name == "vehicle")
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let Ok((Response::FetchVehicleAsset(Ok(vehicle)), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchVehicleAsset { plate }
    ).await else {
        interaction.reply("Error fetching vehicle assests").await;
        return;
    };

    let Some(vehicle) = vehicle else {
        interaction.reply("No vehicle assests found").await;
        return;
    };

    let mut content = format!("**Vehicle**\n\n");
    content.push_str(&format!(
        "**{} - stored: {}**\n",
        vehicle.plate, vehicle.stored
    ));
    interaction.reply(content).await;
}
