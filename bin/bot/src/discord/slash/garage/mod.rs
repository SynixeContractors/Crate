use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandOptionType,
    },
    prelude::Context,
};

use serenity::model::application::interaction::application_command::CommandDataOption;
use synixe_events::garage::db::Response;
use synixe_proc::events_request_2;

use crate::discord::interaction::{Generic, Interaction};

use self::enums::Command;

mod attachment;
pub mod auto_complete;
mod enums;
mod purchase;
mod spawn;

#[allow(clippy::too_many_lines)]
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
                        .name("color")
                        .description("The color of the vehicle")
                        .kind(CommandOptionType::String)
                        .set_autocomplete(true)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("plate")
                        .description("Custom plate for the vehicle")
                        .kind(CommandOptionType::String)
                        .max_length(10)
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
        .create_option(|option| {
            option
                .name("spawn")
                .description("Spawn a vehicle")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("vehicle")
                        .description("The vehicle to spawn")
                        .kind(CommandOptionType::String)
                        .set_autocomplete(true)
                        .required(true)
                })
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand {
        let Some(name) = Command::from_str(subcommand.name.as_str()) else {
            return Ok(());
        };
        return match name {
            Command::View => view(ctx, command, &subcommand.options).await,
            Command::PurchaseVehicle | Command::PurchaseAddon => {
                purchase::purchase(ctx, command, &subcommand.options).await
            }
            Command::Attach => attachment::attach(ctx, command, &subcommand.options).await,
            Command::Detach => attachment::detach(ctx, command, &subcommand.options).await,
            Command::Spawn => spawn::spawn(ctx, command, &subcommand.options).await,
        };
    }
    Ok(())
}

async fn view(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);

    let Ok(Ok((Response::FetchStoredVehicles(Ok(vehicles)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchStoredVehicles{ stored: None, plate: None }
    ).await else {
        return interaction.reply("Error fetching vehicle assests").await;
    };

    if vehicles.is_empty() {
        return interaction.reply("No vehicle assests found").await;
    }

    let mut content = "**Vehicle Assests**\n\n".to_string();
    for vehicle in vehicles {
        let stored = if vehicle.stored {
            "In garage"
        } else {
            "Out in the field"
        };
        #[allow(clippy::uninlined_format_args)]
        content.push_str(&format!(
            "**{}**\nPlate: {}\n{}\n\n",
            vehicle.name, vehicle.plate, stored
        ));
    }
    interaction.reply(content).await
}