use std::fmt::Write;

use serenity::{
    all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};
use synixe_events::garage::db::Response;
use synixe_proc::events_request_2;

use crate::discord::interaction::Interaction;

use self::enums::Command;

mod attachment;
mod enums;
mod purchase;
mod spawn;
mod transport;
pub mod auto_complete;

#[allow(clippy::too_many_lines)]
pub fn register() -> CreateCommand {
    CreateCommand::new("garage")
        .description("Interact with the garage")
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "view",
            "View the garage inventory",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "purchase_addon",
                "purchase an addon asset",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "vehicle",
                    "The vehcile to attach the addon to",
                )
                .set_autocomplete(true)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "addon",
                    "The addon to asset purchase",
                )
                .set_autocomplete(true)
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "purchase_vehicle",
                "purchase a vehicle asset",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "vehicle",
                    "The asset to purchase",
                )
                .set_autocomplete(true)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "color",
                    "The color of the vehicle",
                )
                .set_autocomplete(true)
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "attach",
                "Attach an asset to a vehicle",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "vehicle",
                    "The vehicle in question",
                )
                .set_autocomplete(true)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "addon", "The addon to attach")
                    .set_autocomplete(true)
                    .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "detach",
                "Detach an asset from vehicle",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "vehicle",
                    "The vehicle in question",
                )
                .set_autocomplete(true)
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "spawn", "Spawn a vehicle")
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "vehicle",
                        "The vehicle to spawn",
                    )
                    .set_autocomplete(true)
                    .required(true),
                ),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(options) = &subcommand.value {
        let Some(name) = Command::from_str(subcommand.name.as_str()) else {
            return Ok(());
        };
        return match name {
            Command::View => view(ctx, command, options).await,
            Command::PurchaseVehicle | Command::PurchaseAddon => {
                purchase::purchase(ctx, command, options).await
            }
            Command::Attach => attachment::attach(ctx, command, options).await,
            Command::Detach => attachment::detach(ctx, command, options).await,
            Command::Spawn => spawn::spawn(ctx, command, options).await,
        };
    }
    Ok(())
}

async fn view(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);

    let Ok(Ok((Response::FetchStoredVehicles(Ok(vehicles)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchStoredVehicles {
            stored: None,
            plate: None
        }
    )
    .await
    else {
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
        write!(
            content,
            "**{}**\nPlate: {}\n{}\n\n",
            vehicle.name, vehicle.plate, stored
        )?;
    }
    interaction.reply(content).await
}
