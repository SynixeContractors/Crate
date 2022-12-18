use serenity::{
    model::prelude::application_command::ApplicationCommandInteraction, prelude::Context,
};

use serenity::model::application::interaction::application_command::CommandDataOption;
use synixe_events::garage::db::Response;
use synixe_proc::events_request;

use crate::discord::{self, interaction::Interaction};

pub async fn attach(
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
