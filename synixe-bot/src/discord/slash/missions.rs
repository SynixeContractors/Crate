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
}

pub async fn schedule_run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let subcommand = command.data.options.first().unwrap();
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "new" => new(ctx, command, &subcommand.options).await,
            "upcoming" => upcoming(ctx, command, &subcommand.options).await,
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

#[allow(clippy::unused_async)]
async fn upcoming(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) {
    debug!("upcoming");
}
