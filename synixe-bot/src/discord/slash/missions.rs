use std::time::Duration;

use serenity::{
    builder::CreateApplicationCommand,
    model::{
        application::interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        prelude::{
            application_command::CommandDataOption, command::CommandOptionType,
            component::ButtonStyle, RoleId,
        },
    },
    prelude::*,
};
use synixe_events::missions::db::Response;
use synixe_proc::events_request;
use uuid::Uuid;

pub fn schedule(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("schedule")
        .description("Contract Schedule")
        .create_option(|option| {
            option
                .name("new")
                .description("Add a mission to the schedule")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("month")
                        .description("The month to schedule the mission")
                        .kind(CommandOptionType::Integer)
                        .max_int_value(12)
                        .min_int_value(1)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("day")
                        .description("The day to schedule the mission")
                        .kind(CommandOptionType::Integer)
                        .max_int_value(31)
                        .min_int_value(1)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("hour")
                        .description("The starting hour to schedule the mission")
                        .kind(CommandOptionType::Integer)
                        .max_int_value(23)
                        .min_int_value(0)
                        .required(false)
                })
        })
        .create_option(|option| {
            option
                .name("upcoming")
                .description("View the upcoming missions")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("remove")
                .description("Remove an upcoming mission")
                .kind(CommandOptionType::SubCommand)
        })
}

pub async fn schedule_run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let subcommand = command.data.options.first().unwrap();
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "new" => new(ctx, command, &subcommand.options).await,
            "upcoming" => upcoming(ctx, command, &subcommand.options).await,
            "remove" => remove(ctx, command, &subcommand.options).await,
            _ => unreachable!(),
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn new(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) {
    let date = super::get_datetime(options);
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
                        message
                            .content(format!(
                                "A mission is already scheduled at <t:{}:F>, or the check failed.",
                                date.timestamp()
                            ))
                            .ephemeral(true)
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

async fn upcoming(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _options: &[CommandDataOption],
) {
    match events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await
    {
        Ok(((Response::UpcomingSchedule(Ok(upcoming)), _), _)) => {
            let mut content = String::from("**Upcoming Missions**\n\n");
            for mission in upcoming {
                if let Ok(((Response::FetchMission(Ok(Some(data))), _), _)) = events_request!(
                    bootstrap::NC::get().await,
                    synixe_events::missions::db,
                    FetchMission {
                        mission: mission.mission
                    }
                )
                .await
                {
                    content.push_str(&format!(
                        "**{}**\n<t:{}:F>\n*{}*\n\n",
                        data.name,
                        mission.start_at.timestamp(),
                        data.summary,
                    ));
                }
            }
            command
                .create_interaction_response(&ctx, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|m| m.content(content))
                })
                .await
                .unwrap();
        }
        Ok(_) => {
            command
                .create_interaction_response(&ctx, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|m| {
                            m.content("Failed to fetch upcoming missions")
                                .ephemeral(true)
                        })
                })
                .await
                .unwrap();
        }
        Err(e) => {
            error!("failed to fetch upcoming missions: {}", e);
            command
                .create_interaction_response(&ctx, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|m| {
                            m.content(format!("Failed to fetch upcoming missions: {}", e))
                                .ephemeral(true)
                        })
                })
                .await
                .unwrap();
        }
    }
}

#[allow(clippy::too_many_lines)]
pub async fn remove(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _options: &[CommandDataOption],
) {
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
    debug!("fetching missions");
    command
        .create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|m| m.content("Fetching missions...").ephemeral(true))
        })
        .await
        .unwrap();
    if let Ok(((Response::UpcomingSchedule(Ok(missions)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await
    {
        let m = command
            .create_followup_message(&ctx, |r| {
                r.content("Select Mission").ephemeral(true).components(|c| {
                    c.create_action_row(|r| {
                        r.create_select_menu(|m| {
                            m.custom_id("mission").options(|o| {
                                for mission in missions {
                                    o.create_option(|o| {
                                        o.label(format!(
                                            "{} - <t:{}:f>",
                                            mission.mission,
                                            mission.start_at.timestamp()
                                        ))
                                        .value(mission.id)
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
            if let Some(mission_id) = interaction.data.values.get(0) {
                let mission_id: Uuid = mission_id.parse().unwrap();
                interaction
                    .create_interaction_response(&ctx, |r| {
                        r.kind(InteractionResponseType::DeferredUpdateMessage)
                    })
                    .await
                    .unwrap();
                let m = interaction
                    .create_followup_message(&ctx, |r| {
                        r.content(format!("Are you sure you want to remove `{}`?", mission_id))
                            .ephemeral(true)
                            .components(|c| {
                                c.create_action_row(|r| {
                                    r.create_button(|b| {
                                        b.style(ButtonStyle::Danger).label("Yes").custom_id("yes")
                                    })
                                    .create_button(|b| {
                                        b.style(ButtonStyle::Primary).label("No").custom_id("no")
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
                    if interaction.data.custom_id == "yes" {
                        debug!("removing mission");
                        interaction
                            .create_interaction_response(&ctx, |r| {
                                r.kind(InteractionResponseType::DeferredUpdateMessage)
                            })
                            .await
                            .unwrap();
                        if let Ok(((Response::Unschedule(Ok(())), _), _)) = events_request!(
                            bootstrap::NC::get().await,
                            synixe_events::missions::db,
                            Unschedule {
                                scheduled_mission: mission_id
                            }
                        )
                        .await
                        {
                            interaction
                                .edit_followup_message(&ctx, m.id, |r| {
                                    r.content(format!("Removed `{}`", mission_id))
                                        .components(|c| c)
                                })
                                .await
                                .unwrap();
                        } else {
                            interaction
                                .edit_followup_message(&ctx, m.id, |r| {
                                    r.content(format!("Failed to remove `{}`", mission_id))
                                        .components(|c| c)
                                })
                                .await
                                .unwrap();
                        }
                    } else {
                        interaction
                            .edit_followup_message(&ctx, m.id, |r| {
                                r.content("Cancelled mission removal").components(|c| c)
                            })
                            .await
                            .unwrap();
                    }
                } else {
                    interaction
                        .edit_followup_message(&ctx, m.id, |r| {
                            r.content("Timed out").components(|c| c)
                        })
                        .await
                        .unwrap();
                }
            } else {
                interaction
                    .edit_followup_message(&ctx, m.id, |r| {
                        r.content("No mission selected").components(|c| c)
                    })
                    .await
                    .unwrap();
            }
        } else {
            command
                .edit_original_interaction_response(&ctx, |r| {
                    r.content("Timed out").components(|c| c)
                })
                .await
                .unwrap();
        }
    } else {
        command
            .edit_original_interaction_response(&ctx, |r| {
                r.content("Failed to fetch missions").components(|c| c)
            })
            .await
            .unwrap();
    }
}
