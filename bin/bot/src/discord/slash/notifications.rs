use serenity::{
    all::{CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};
use synixe_events::discord::db::{Notifications, Response};
use synixe_proc::events_request_5;

use crate::{discord::interaction::Interaction, get_option};

pub fn register() -> CreateCommand {
    CreateCommand::new("notifications")
        .description("Choose which direct messages Brodsky sends you, or see your current choices")
        .add_option(CreateCommandOption::new(
            CommandOptionType::Boolean,
            "payday",
            "Get a message when you're paid for a mission",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::Boolean,
            "mission_reminders",
            "Get a message 15 minutes before a mission you RSVP'd yes to, if you're not on the server",
        ))
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let options = &command.data.options;
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    let payday = get_option!(options, "payday", Boolean).copied();
    let mission_reminder = get_option!(options, "mission_reminders", Boolean).copied();
    let member = command.user.id;

    let notifications = if payday.is_none() && mission_reminder.is_none() {
        match events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::discord::db,
            GetNotifications { member }
        )
        .await
        {
            Ok(Ok((Response::GetNotifications(Ok(notifications)), _))) => notifications,
            _ => {
                return interaction
                    .reply("Failed to fetch your notifications")
                    .await;
            }
        }
    } else {
        match events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::discord::db,
            SetNotifications {
                member,
                payday,
                mission_reminder,
            }
        )
        .await
        {
            Ok(Ok((Response::SetNotifications(Ok(notifications)), _))) => notifications,
            _ => {
                return interaction
                    .reply("Failed to update your notifications")
                    .await;
            }
        }
    };
    interaction.reply(describe(notifications)).await
}

fn describe(notifications: Notifications) -> String {
    let state = |on: bool| if on { "**On**" } else { "Off" };
    format!(
        "Your notifications:\n- Payday: {}\n- Mission reminders: {}\n-# Change them with `/notifications`, e.g. `/notifications payday:True`",
        state(notifications.payday),
        state(notifications.mission_reminder),
    )
}
