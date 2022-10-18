use std::time::Duration;

use chrono::{Datelike, TimeZone};
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

pub fn schedule(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("schedule")
        .description("Schedule a mission")
        .create_option(|option| {
            option
                .name("mission")
                .description("The mission to schedule")
                .kind(CommandOptionType::String)
                .required(true)
        })
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
    let mut options = ScheduleOptions::default();
    command
        .data
        .options
        .iter()
        .for_each(|option| match option.name.as_str() {
            "mission" => {
                options.mission = option.value.as_ref().unwrap().as_str().unwrap().to_string();
            }
            "day" => options.day = option.value.as_ref().unwrap().as_str().unwrap().to_string(),
            _ => {}
        });
    let (month, day) = options.day.split_once('-').unwrap();
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
    debug!("schedule {} on {}", options.mission, date);
    if command
        .member
        .as_ref()
        .unwrap()
        .roles
        .contains(&RoleId(1_020_252_253_287_886_858))
    {
        debug!("fetching mission");
        command
            .create_interaction_response(&ctx, |response| {
                response
                    .kind(InteractionResponseType::DeferredChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content("Fetching mission...").ephemeral(true)
                    })
            })
            .await
            .unwrap();
        debug!("sending confirmation");
        let m = command
            .create_followup_message(&ctx, |r| {
                r.content(format!("Schedule `{}` for `{}`?", options.mission, date))
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

        command
            .edit_followup_message(&ctx, m.id, |r| r.components(|c| c).content("Processing..."))
            .await
            .unwrap();

        debug!("interaction received");
        if let Some(interaction) = interaction {
            interaction
                .edit_followup_message(&ctx, m.id, |r| r.ephemeral(true).content("Scheduled"))
                .await
                .unwrap();
        } else {
            command
                .edit_followup_message(&ctx, m.id, |r| {
                    r.ephemeral(true).content("Timed out, scheduling cancelled")
                })
                .await
                .unwrap();
        }
    } else {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content("You can't do that."))
            })
            .await
            .unwrap();
    }
}

#[derive(Default)]
struct ScheduleOptions {
    mission: String,
    day: String,
}
