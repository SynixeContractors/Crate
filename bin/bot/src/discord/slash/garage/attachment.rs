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

    let addon = options
        .iter()
        .find(|option| option.name == "addon")
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
        .parse()
        .unwrap();

    let Ok(Ok((Response::AttachAddon(Ok(())), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        AttachAddon {
            plate,
            addon,
            member: command.member.as_ref().unwrap().user.id
        }
    )
    .await
    else {
        interaction.reply("Error attaching addon").await;
        return;
    };

    interaction.reply("**Addon Attached**\n\n").await;
}

pub async fn detach(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) {
    let mut interaction =
        Interaction::new(ctx, discord::interaction::Generic::Application(command));
    interaction.reply("This is the garage detach command").await;

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

    let Ok(Ok((Response::DetachAddon(Ok(())), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        DetachAddon {
            plate,
            member: command.member.as_ref().unwrap().user.id
        }
    )
    .await
    else {
        interaction.reply("Error detaching addon").await;
        return;
    };

    interaction.reply("**Addon Detached**\n\n").await;
}
