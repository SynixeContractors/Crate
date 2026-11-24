use serenity::{
    all::{CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};

use self::enums::Command;

mod attachment;
pub mod auto_complete;
mod enums;
pub mod info;
mod purchase;
mod spawn;

#[allow(clippy::too_many_lines)]
pub fn register() -> CreateCommand {
    CreateCommand::new("garage")
        .description("Interact with the garage")
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "list",
            "List every vehicle with its fuel and damage",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "view",
                "View a vehicle's fuel, damage and repair costs",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "vehicle", "The vehicle")
                    .set_autocomplete(true)
                    .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "repair",
                "Repair part of a stored vehicle, paid for by the company",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "vehicle",
                    "The vehicle, which has to be in the garage",
                )
                .set_autocomplete(true)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "part", "The part to repair")
                    .set_autocomplete(true)
                    .required(true),
            ),
        )
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
            Command::List => info::list(ctx, command, options).await,
            Command::View => info::view(ctx, command, options).await,
            Command::Repair => info::repair(ctx, command, options).await,
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
