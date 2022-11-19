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

use crate::discord::{interaction::Interaction, self};

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
                .name("modify")
                .description("Modify an asset")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("attach")
                        .description("Attach an asset to a vehicle")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("dettach")
                        .description("Dettach an asset from a vehicle")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
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
            // "modify" => modify(ctx, command, &subcommand.options).await,
            _ => unreachable!(),
        }
    }
}

async fn view(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _options: &[CommandDataOption],
) {
    let mut interaction = Interaction::new(ctx, discord::interaction::Generic::Application(command));
    interaction.reply("This is the garage view command").await;

    let Ok((Response::FetchVehicleAssets(Ok(vehicles)), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchVehicleAssets{ stored: None}
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
