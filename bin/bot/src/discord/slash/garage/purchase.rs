use serenity::{
    model::prelude::application_command::ApplicationCommandInteraction, prelude::Context,
};

use serenity::model::application::interaction::application_command::CommandDataOption;
use synixe_events::garage::db::Response;
use synixe_proc::events_request;
use uuid::Uuid;

use crate::discord::{self, interaction::Interaction};
use crate::get_option;

pub async fn purchase(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction =
        Interaction::new(ctx, discord::interaction::Generic::Application(command));

    let plate = get_option!(options, "vehicle", String);

    let kind = options
        .iter()
        .rev()
        .find(|option| option.name == "vehicle" || option.name == "addon");

    let Some(kind) = kind else {
        return interaction.reply("Required option not provided: vehicle or addon").await;
    };

    let Some(id) = get_option!(options, &kind.name, String) else {
        return interaction.reply("Required option not provided: vehicle or addon").await;
    };
    let Ok(id) = Uuid::parse_str(id.as_str()) else {
        return interaction.reply("Invalid vehicle or addon UUID").await;
    };

    let Ok(Ok((Response::PurchaseShopAsset(Ok(())), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        PurchaseShopAsset {
            id,
            plate: plate.cloned(),
            member: command.member.as_ref().expect("member should always exist on guild commands").user.id
        }
    ).await else {
        return interaction.reply("Error purchasing asset").await;
    };

    interaction.reply("**Asset Purchase**\n\n").await
}
