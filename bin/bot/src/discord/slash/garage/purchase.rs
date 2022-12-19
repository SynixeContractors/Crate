use serenity::{
    model::prelude::application_command::ApplicationCommandInteraction, prelude::Context,
};

use serenity::model::application::interaction::application_command::CommandDataOption;
use synixe_events::garage::db::Response;
use synixe_proc::events_request;

use crate::discord::{self, interaction::Interaction};

pub async fn purchase(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) {
    let mut interaction =
        Interaction::new(ctx, discord::interaction::Generic::Application(command));
    interaction
        .reply("This is the garage purchase command")
        .await;

    let plate = options
        .iter()
        .find(|option| option.name == "plate")
        .and_then(|p| p.value.as_ref().unwrap().as_str().map(str::to_string));

    let id = options
        .iter()
        .rev()
        .find(|option| option.name == "vehicle" || option.name == "addon")
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let Ok(Ok((Response::PurchaseShopAsset(Ok(())), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        PurchaseShopAsset{ id: id.parse().unwrap(), plate, member: command.member.as_ref().unwrap().user.id}
    ).await else {
        interaction.reply("Error purchasing asset").await;
        return;
    };

    interaction.reply("**Asset Purchase**\n\n").await;
}
