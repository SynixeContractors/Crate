use serenity::all::{CommandDataOption, CommandInteraction};
use serenity::client::Context;
use synixe_events::garage::db::Response;
use synixe_meta::discord::role::LEADERSHIP;
use synixe_proc::events_request_2;
use uuid::Uuid;

use crate::discord::interaction::Interaction;
use crate::discord::slash::ShouldAsk;
use crate::get_option;

pub async fn attach(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    super::super::requires_roles(
        command.user.id,
        &[LEADERSHIP],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Yes(("garage attach", options)),
        &mut interaction,
    )
    .await?;

    let Some(plate) = get_option!(options, "vehicle", String) else {
        return interaction
            .reply("Required option not provided: vehicle")
            .await;
    };

    let Some(addon) = get_option!(options, "addon", String) else {
        return interaction
            .reply("Required option not provided: addon")
            .await;
    };
    let Ok(addon) = Uuid::parse_str(addon.as_str()) else {
        return interaction.reply("Invalid addon UUID").await;
    };

    let Ok(Ok((Response::AttachAddon(Ok(())), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        AttachAddon {
            plate: plate.clone(),
            addon,
            member: command
                .member
                .as_ref()
                .expect("member should always exist on guild commands")
                .user
                .id
        }
    )
    .await
    else {
        return interaction.reply("Error attaching addon").await;
    };

    interaction.reply("**Addon Attached**\n\n").await
}

pub async fn detach(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    super::super::requires_roles(
        command.user.id,
        &[LEADERSHIP],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Yes(("garage detach", options)),
        &mut interaction,
    )
    .await?;

    let Some(plate) = get_option!(options, "vehicle", String) else {
        return interaction
            .reply("Required option not provided: vehicle")
            .await;
    };

    let Ok(Ok((Response::DetachAddon(Ok(())), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        DetachAddon {
            plate: plate.clone(),
            member: command
                .member
                .as_ref()
                .expect("member should always exist on guild commands")
                .user
                .id
        }
    )
    .await
    else {
        return interaction.reply("Error detaching addon").await;
    };
    interaction.reply("**Addon Detached**\n\n").await
}
