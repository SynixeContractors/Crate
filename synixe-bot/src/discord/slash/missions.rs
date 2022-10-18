use std::time::Duration;

use chrono::{Datelike, NaiveDateTime, TimeZone};
use chrono_tz::America::New_York;
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        application::interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        prelude::{command::CommandOptionType, component::ButtonStyle, RoleId},
    },
    prelude::*,
};
use synixe_events::missions::db::Response;
use synixe_proc::events_request;

pub fn schedule(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("schedule")
        .description("Schedule a mission")
        .create_option(|option| {
            option
                .name("day")
                .description("The day to schedule the mission")
                .kind(CommandOptionType::String)
                .required(true)
        })
}

#[allow(clippy::too_many_lines)]
pub async fn schedule_run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let date = get_date(command);
    if !super::requires_role(
        RoleId(1_020_252_253_287_886_858),
        &command.member.as_ref().unwrap().roles,
        ctx,
        command,
    )
    .await
    {
        return;
    }
    if let Ok(((Response::IsScheduled(Ok(Some(false) | None)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        IsScheduled { date }
    )
    .await
    {
        debug!("No mission scheduled for {}", date);
    } else {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content(
                            "A mission is already scheduled for that day, or the check failed.",
                        )
                    })
            })
            .await
            .unwrap();
        return;
    }
    debug!("fetching missions");
    command
        .create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|m| m.content("Fetching missions...").ephemeral(true))
        })
        .await
        .unwrap();
    if let Ok(((Response::FetchMissionList(Ok(missions)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionList {}
    )
    .await
    {
        let m = command
            .create_followup_message(&ctx, |r| {
                r.content(format!("Select mission for <t:{}:F>", date.timestamp()))
                    .ephemeral(true)
                    .components(|c| {
                        c.create_action_row(|r| {
                            r.create_select_menu(|m| {
                                m.custom_id("mission").options(|o| {
                                    for mission in missions {
                                        o.create_option(|o| {
                                            o.label(mission.name).value(mission.id.to_string())
                                        });
                                    }
                                    o
                                })
                            })
                        })
                    })
            })
            .await
            .unwrap();
        if let Some(interaction) = m
            .await_component_interaction(&ctx)
            .timeout(Duration::from_secs(60 * 3))
            .collect_limit(1)
            .await
        {
            debug!("sending confirmation");
            let mission_id = interaction.data.values.first().unwrap();
            command
                .edit_followup_message(&ctx, m.id, |r| r.content(mission_id).components(|c| c))
                .await
                .unwrap();
            let m = command
                .create_followup_message(&ctx, |r| {
                    r.content(format!(
                        "Schedule `{}` for <t:{}:F>?",
                        mission_id,
                        date.timestamp()
                    ))
                    .ephemeral(true)
                    .components(|c| {
                        c.create_action_row(|r| {
                            r.create_button(|b| {
                                b.label("Confirm")
                                    .custom_id("confirm")
                                    .emoji('✅')
                                    .style(ButtonStyle::Primary)
                            })
                            .create_button(|b| {
                                b.label("Cancel")
                                    .custom_id("cancel")
                                    .emoji('❌')
                                    .style(ButtonStyle::Danger)
                            })
                        })
                    })
                })
                .await
                .unwrap();
            debug!("waiting for confirmation");
            let interaction = m
                .await_component_interaction(&ctx)
                .timeout(Duration::from_secs(60 * 3))
                .collect_limit(1)
                .await;

            debug!("interaction received");
            if let Some(interaction) = interaction {
                if interaction.data.custom_id == "confirm" {
                    if let Err(e) = events_request!(
                        bootstrap::NC::get().await,
                        synixe_events::missions::db,
                        Schedule {
                            mission: mission_id.to_string(),
                            date
                        }
                    )
                    .await
                    {
                        error!("failed to schedule mission: {}", e);
                        command
                            .edit_followup_message(&ctx, m.id, |r| {
                                r.content(format!("Failed to schedule mission: {}", e))
                            })
                            .await
                            .unwrap();
                    } else {
                        command
                            .edit_followup_message(&ctx, m.id, |r| {
                                r.content(format!(
                                    "Scheduled `{}` for <t:{}:F>",
                                    mission_id,
                                    date.timestamp()
                                ))
                                .components(|c| c)
                            })
                            .await
                            .unwrap();
                    }
                } else {
                    command
                        .edit_followup_message(&ctx, m.id, |r| {
                            r.content("Cancelled").components(|c| c)
                        })
                        .await
                        .unwrap();
                }
            } else {
                command
                    .edit_followup_message(&ctx, m.id, |r| {
                        r.ephemeral(true)
                            .content("Timed out, scheduling cancelled")
                            .components(|c| c)
                    })
                    .await
                    .unwrap();
            }
        } else {
            command
                .edit_followup_message(&ctx, m.id, |r| {
                    r.ephemeral(true)
                        .content("Timed out, scheduling cancelled")
                        .components(|c| c)
                })
                .await
                .unwrap();
        }
    } else {
        command
            .create_followup_message(&ctx, |r| {
                r.content("Failed to fetch missions").ephemeral(true)
            })
            .await
            .unwrap();
    }
}

fn get_date(command: &ApplicationCommandInteraction) -> NaiveDateTime {
    let day = command
        .data
        .options
        .iter()
        .find(|option| option.name == "day")
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let (month, day) = day.split_once('-').unwrap();
    debug!("month: {}, day: {}", month, day);
    let date = {
        let year = chrono::Utc::now().with_timezone(&New_York).date().year();
        let possible = New_York
            .ymd(year, month.parse().unwrap(), day.parse().unwrap())
            .and_hms(22, 0, 0);
        if chrono::Utc::now()
            .signed_duration_since(possible)
            .num_seconds()
            > 0
        {
            possible.with_year(year + 1).unwrap()
        } else {
            possible
        }
    };
    date.naive_utc()
}
