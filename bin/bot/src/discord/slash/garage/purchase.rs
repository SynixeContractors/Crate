use std::sync::Arc;

use serenity::{
    model::prelude::{
        application_command::ApplicationCommandInteraction, autocomplete::AutocompleteInteraction,
    },
    prelude::Context,
};

use serenity::model::application::interaction::application_command::CommandDataOption;
use synixe_events::garage::db::Response;
use synixe_proc::events_request;

use crate::discord::{self, interaction::Interaction};

use super::enums::GarageSubCommands;

pub async fn purchase_autocomplete(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    options: &[CommandDataOption],
) {
    let focus = options.iter().find(|o| o.focused);
    let Some(focus) = focus else {
        return;
    };
    match GarageSubCommands::from_str(focus.name.as_str()).unwrap() {
        GarageSubCommands::Vehicle => autocomplete_shop_assets(ctx, autocomplete, &focus, true).await,
        GarageSubCommands::Addon => autocomplete_shop_assets(ctx, autocomplete, &focus, false).await,
        _ => unreachable!(),
    }
}

async fn autocomplete_shop_assets(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    focus: &CommandDataOption,
    vic: bool,
) {
    let Ok((Response::FetchAllShopAssests(Ok(mut assets)), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchAllShopAssests { search: Some(focus.value.as_ref().unwrap().as_str().unwrap().to_string()) }
    ).await else {
        error!("Failed to fetch all shop assests");
        return;
    };

    assets = match vic {
        true => assets.into_iter().filter(|a| a.base.is_none()).collect(),
        false => assets.into_iter().filter(|a| a.base.is_some()).collect(),
    };

    if assets.len() > 25 {
        assets.truncate(25);
    }
    if let Err(e) = autocomplete
        .create_autocomplete_response(&ctx.http, |f| {
            for assets in assets {
                f.add_string_choice(format!("{} - ${}", assets.name, assets.cost), assets.id);
            }
            f
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
}

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
        .find(|option| option.name == "plate" || option.name == "addon")
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap_or(&format!("addon-{}", uuid::Uuid::new_v4()))
        .to_string();

    let id = options
        .iter()
        .find(|option| option.name == "vehicle" || option.name == "addon")
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let Ok((Response::PurchaseVehicleAsset(Ok(())), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        PurchaseVehicleAsset{ id: id.parse().unwrap(), plate: plate.clone(), member: command.member.as_ref().unwrap().user.id}
    ).await else {
        interaction.reply("Error purchasing asset").await;
        return;
    };

    interaction.reply(format!("**Asset Purchase: {}**\n\n", plate.clone())).await;
}
