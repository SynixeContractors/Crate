//! Reports on members who've stopped showing up, with a button for staff to reach out to them

use serenity::{
    all::{ButtonStyle, ComponentInteraction, ReactionType, UserId},
    builder::{
        CreateActionRow, CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage,
        CreateMessage, EditInteractionResponse,
    },
    http::CacheHttp,
    prelude::Context,
};
use synixe_events::discord::db::{MemberActivity, Response};
use synixe_meta::discord::{GUILD, channel::LOG, role::STAFF};
use synixe_proc::events_request_5;

/// Prefix of the Reach out button on an activity report, followed by the member's ID
pub const REACHOUT_SEND: &str = "reachout_send:";

/// The message sent to a member staff reach out to. Its buttons are handled with the other `reachout_`
/// buttons in the interaction handler.
const REACHOUT_MESSAGE: &str = "Hey! It's been a long while since you've played Arma 3 with Synixe Contractors. We're reaching out to see if you're still wanting to be a member of our Discord server. We'd love to see you return in-game, but we understand if you've moved on. Please select one of the following options:\n\n🔫 I am currently busy or not interested in playing Arma, but I want to return in-game with Synixe in the future!\n👀 I am not interested in playing, but I wish to remain in the Discord for now\n👋 I am no longer interested in being a part of Synixe Contractors and would like to leave the Discord server\n\nIf you have any questions or concerns, please feel free to reach out to a staff member. We hope to see you back in the future! You can always find us at <https://synixe.contractors>";

fn when(at: Option<time::OffsetDateTime>) -> String {
    at.map_or_else(
        || String::from("never"),
        |at| format!("<t:{}:R>", at.unix_timestamp()),
    )
}

/// Posts a report in #log on a member who just lost the active role
pub async fn report_inactive(http: impl CacheHttp, member: UserId) {
    let activity = if let Ok(Ok((Response::MemberActivity(Ok(activity)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::discord::db,
        MemberActivity { member }
    )
    .await
    {
        activity
    } else {
        error!("failed to fetch activity for {member}");
        MemberActivity::default()
    };
    let reachout = activity.last_reachout.map_or_else(String::new, |at| {
        format!("\n-# Staff last reached out <t:{}:R>", at.unix_timestamp())
    });
    if let Err(e) = LOG
        .send_message(
            http,
            CreateMessage::default()
                .content(format!(
                    "<@{member}> is no longer active\n{} missions · last mission {} · last on a server {}{reachout}",
                    activity.missions,
                    when(activity.last_mission),
                    when(activity.last_connected),
                ))
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new(format!("{REACHOUT_SEND}{member}"))
                        .style(ButtonStyle::Secondary)
                        .label("Reach out"),
                ])]),
        )
        .await
    {
        error!("failed to post activity report for {member}: {e}");
    }
}

/// Handles the Reach out button on an activity report: sends the member the reachout message
pub async fn reachout_button(
    ctx: &Context,
    component: &ComponentInteraction,
) -> serenity::Result<()> {
    let Some(member) = component
        .data
        .custom_id
        .strip_prefix(REACHOUT_SEND)
        .and_then(|id| id.parse::<u64>().ok())
        .filter(|id| *id != 0)
        .map(UserId::new)
    else {
        return Ok(());
    };
    let staff = component.user.id;
    let is_staff = GUILD
        .member(ctx, staff)
        .await
        .is_ok_and(|m| m.roles.contains(&STAFF));
    if !is_staff {
        return component
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Only staff can reach out to members")
                        .ephemeral(true),
                ),
            )
            .await;
    }

    // The button is removed first, so two staff can't both send it
    let report = component.message.content.clone();
    component
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(format!("{report}\n:hourglass: Reaching out..."))
                    .components(vec![]),
            ),
        )
        .await?;

    let sent = match member.create_dm_channel(ctx).await {
        Ok(channel) => channel
            .send_message(
                &ctx.http,
                CreateMessage::default()
                    .content(REACHOUT_MESSAGE)
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("reachout_yes")
                            .style(ButtonStyle::Secondary)
                            .emoji(ReactionType::Unicode("🔫".to_string())),
                        CreateButton::new("reachout_maybe")
                            .style(ButtonStyle::Secondary)
                            .emoji(ReactionType::Unicode("👀".to_string())),
                        CreateButton::new("reachout_no")
                            .style(ButtonStyle::Secondary)
                            .emoji(ReactionType::Unicode("👋".to_string())),
                    ])]),
            )
            .await
            .map_err(|e| error!("failed to send reachout to {member}: {e}"))
            .is_ok(),
        Err(e) => {
            error!("unable to create dm channel for {member}: {e}");
            false
        }
    };

    let outcome = if sent {
        if !matches!(
            events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::discord::db,
                RecordReachout {
                    member,
                    sent_by: staff
                }
            )
            .await,
            Ok(Ok((Response::RecordReachout(Ok(())), _)))
        ) {
            error!("failed to record reachout to {member}");
        }
        format!(":envelope: Reached out by <@{staff}>")
    } else {
        format!(":warning: <@{staff}> tried to reach out, but they don't accept direct messages")
    };
    component
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new().content(format!("{report}\n{outcome}")),
        )
        .await?;
    Ok(())
}
