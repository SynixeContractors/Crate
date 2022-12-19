use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandOptionType,
    },
    prelude::Context,
};

use serenity::model::application::interaction::application_command::CommandDataOption;
use synixe_events::garage::db::Response;
use synixe_proc::events_request;

use crate::discord::{self, interaction::Interaction};

use self::enums::Command;
// use super::enums::{GarageCommands, GarageSubCommands};

mod attachment;
pub mod auto_complete;
mod enums;
mod purchase;

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
                .name("purchase_addon")
                .description("purchase an addon asset")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("vehicle")
                        .description("The vehcile to attach the addon to")
                        .kind(CommandOptionType::String)
                        .set_autocomplete(true)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("addon")
                        .description("The addon to asset purchase")
                        .kind(CommandOptionType::String)
                        .set_autocomplete(true)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("purchase_vehicle")
                .description("purchase a vehicle asset")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("vehicle")
                        .description("The asset to purchase")
                        .kind(CommandOptionType::String)
                        .set_autocomplete(true)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("plate")
                        .description("Custom plate for the vehicle")
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
                        .set_autocomplete(true)
                        .required(true)
                })
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let subcommand = command.data.options.first().unwrap();
    if subcommand.kind == CommandOptionType::SubCommand {
        match Command::from_str(subcommand.name.as_str()).unwrap() {
            Command::View => view(ctx, command, &subcommand.options).await,
            Command::PurchaseVehicle | Command::PurchaseAddon => {
                purchase::purchase(ctx, command, &subcommand.options).await;
            }
            Command::Attach => attachment::attach(ctx, command, &subcommand.options).await,
            Command::Detach => attachment::detach(ctx, command, &subcommand.options).await,
        }
    }
}

async fn view(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _options: &[CommandDataOption],
) {
    let mut interaction =
        Interaction::new(ctx, discord::interaction::Generic::Application(command));
    interaction.reply("This is the garage view command").await;

    let Ok(Ok((Response::FetchStoredVehicles(Ok(vehicles)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchStoredVehicles{ stored: None, plate: None }
    ).await else {
        interaction.reply("Error fetching vehicle assests").await;
        return;
    };

    if vehicles.is_empty() {
        interaction.reply("No vehicle assests found").await;
        return;
    }

    let mut content = "**Vehicle Assests**\n\n".to_string();
    for vehicle in vehicles {
        content.push_str(&format!(
            "**{} - stored: {}**\n",
            vehicle.plate, vehicle.stored
        ));
    }
    interaction.reply(content).await;
}
