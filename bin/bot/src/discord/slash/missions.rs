use std::time::Duration;

use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        prelude::{
            application_command::CommandDataOption, autocomplete::AutocompleteInteraction,
            command::CommandOptionType, component::ButtonStyle,
            message_component::MessageComponentInteraction, modal::ModalSubmitInteraction,
            InteractionResponseType, MessageId, ReactionType,
        },
    },
    prelude::*,
};
use synixe_events::missions::db::Response;
use synixe_meta::discord::{
    channel::{BOT, SCHEDULE},
    role::MISSION_REVIEWER,
};
use synixe_model::missions::{Mission, MissionRsvp, Rsvp, ScheduledMission};
use synixe_proc::events_request;
use time::format_description::{self, well_known::Rfc3339};
use time_tz::{timezones::db::america::NEW_YORK, OffsetDateTimeExt};

use crate::discord::interaction::{Confirmation, Interaction};

const TIME_FORMAT: &str =
    "[year]-[month]-[day] [hour]:[minute] [offset_hour sign:mandatory]:[offset_minute]";

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
                        .name("mission")
                        .description("Mission to schedule")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
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
        .create_option(|option| {
            option
                .name("post")
                .description("Post the upcoming mission")
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
            "post" => post(ctx, command, &subcommand.options).await,
            _ => unreachable!(),
        }
    }
}

pub async fn schedule_autocomplete(ctx: &Context, autocomplete: &AutocompleteInteraction) {
    let subcommand = autocomplete.data.options.first().unwrap();
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "new" => new_autocomplete(ctx, autocomplete, &subcommand.options).await,
            _ => (),
        }
    }
}

#[allow(clippy::too_many_lines)]
pub async fn component(ctx: &Context, component: &MessageComponentInteraction) {
    let message = component.message.id;
    let Ok(((Response::FetchScheduledMission(Ok(Some(scheduled))), _), _)) =
        events_request!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            FetchScheduledMission { message }
        )
        .await else {
            error!("Failed to fetch scheduled mission for component");
            return;
        };
    let Ok(((Response::FetchMission(Ok(Some(mission))), _), _)) =
        events_request!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            FetchMission { mission: scheduled.mission.clone() }
        )
        .await else {
            error!("Failed to fetch mission for component");
            return;
        };
    match component.data.custom_id.as_str() {
        "yes" => {
            let Ok(((Response::AddMissionRsvp(Ok(())), _), _)) =
                events_request!(
                    bootstrap::NC::get().await,
                    synixe_events::missions::db,
                    AddMissionRsvp {
                        mission: scheduled.mission,
                        member: component.user.id.to_string(),
                        rsvp: Rsvp::Yes,
                        details: None,
                    }
                )
                .await else {
                    error!("Failed to add mission rsvp for component");
                    return;
                };
        }
        "maybe" => {
            if let Err(e) = component
                .create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::Modal)
                        .interaction_response_data(|d| {
                            d.custom_id("rsvp_maybe")
                                .content("Please provide details for your maybe RSVP")
                                .components(|c| {
                                    c.create_action_row(|r| {
                                        r.create_select_menu(|s| {
                                            s.custom_id("maybe_reason").options(|o| {
                                                o.create_option(|o| {
                                                    o.label("I'm not sure if I can make it")
                                                        .value("not_sure")
                                                })
                                                .create_option(|o| {
                                                    o.label("I'm not interested in this mission")
                                                        .value("not_interested")
                                                })
                                                .create_option(|o| {
                                                    o.label("I'm burnt out and may not want to")
                                                        .value("burnt_out")
                                                })
                                                .create_option(|o| o.label("Other").value("other"))
                                            })
                                        })
                                        .create_input_text(|i| {
                                            i.custom_id("maybe_details")
                                                .placeholder("Details")
                                                .required(false)
                                        })
                                    })
                                })
                        })
                })
                .await
            {
                error!("Failed to create interaction response for component: {}", e);
            }
        }
        "no" => {
            if let Err(e) = component
                .create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::Modal)
                        .interaction_response_data(|d| {
                            d.custom_id("rsvp_no")
                                .content("Please provide details for your no RSVP")
                                .components(|c| {
                                    c.create_action_row(|r| {
                                        r.create_select_menu(|s| {
                                            s.custom_id("no_reason").options(|o| {
                                                o.create_option(|o| {
                                                    o.label("I won't be able to make it")
                                                        .value("not_sure")
                                                })
                                                .create_option(|o| {
                                                    o.label("I'm not interested in this mission")
                                                        .value("not_interested")
                                                })
                                                .create_option(|o| {
                                                    o.label("I'm burnt out").value("burnt_out")
                                                })
                                            })
                                        })
                                        .create_input_text(|i| {
                                            i.custom_id("maybe_details")
                                                .placeholder("Details")
                                                .required(false)
                                        })
                                    })
                                })
                        })
                })
                .await
            {
                error!("Failed to create interaction response for component: {}", e);
            }
        }
        _ => {
            warn!("Unknown component id: {}", component.data.custom_id);
        }
    }
}

#[allow(clippy::unused_async)]
pub async fn rsvp_modal(ctx: &Context, modal: &ModalSubmitInteraction) {
    println!("{:?}", modal.data.components[0]);
}

#[allow(clippy::too_many_lines)]
async fn new(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) {
    let mut interaction = Interaction::new(ctx, command);
    let date = super::get_datetime(options);
    super::requires_role(
        MISSION_REVIEWER,
        &command.member.as_ref().unwrap().roles,
        &mut interaction,
    )
    .await;
    if let Ok(((Response::IsScheduled(Ok(Some(Some(false) | None) | None)), _), _)) =
        events_request!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            IsScheduled { date }
        )
        .await
    {
        debug!("No mission scheduled for {}", date);
    } else {
        interaction
            .reply(format!(
                "A mission is already scheduled at <t:{}:F>, or the check failed.",
                date.unix_timestamp()
            ))
            .await;
        return;
    }
    let mission_id = options
        .iter()
        .find(|option| option.name == "mission")
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap();
    let Ok(((Response::FetchMissionList(Ok(missions)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionList {
            search: Some(mission_id.to_string()),
        }
    )
    .await else {
        error!("failed to fetch mission list");
        return;
    };
    if missions.len() != 1 {
        interaction
            .reply(format!(
                "Found {} missions matching `{}`",
                missions.len(),
                mission_id
            ))
            .await;
        return;
    }
    let confirm = interaction
        .confirm(&format!(
            "Schedule `{}` for <t:{}:F>?",
            mission_id,
            date.unix_timestamp()
        ))
        .await;
    match confirm {
        Confirmation::Yes => {
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
                interaction
                    .reply(format!("Failed to schedule mission: {e}"))
                    .await;
            } else {
                interaction
                    .reply(format!(
                        "Scheduled `{}` for <t:{}:F>",
                        mission_id,
                        date.unix_timestamp()
                    ))
                    .await;
            }
        }
        Confirmation::No => {
            interaction.reply("Cancelled.").await;
        }
        Confirmation::Timeout => {}
    }
}

async fn new_autocomplete(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    options: &[CommandDataOption],
) {
    let focus = options.iter().find(|o| o.focused);
    let Some(focus) = focus else {
        return;
    };
    if focus.name != "mission" {
        return;
    }
    let Ok(((Response::FetchMissionList(Ok(mut missions)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionList {
            search: Some(focus.value.as_ref().unwrap().as_str().unwrap().to_string())
        }
    )
    .await else {
        error!("failed to fetch mission list");
        return;
    };
    if missions.len() > 25 {
        missions.truncate(25);
    }
    if let Err(e) = autocomplete
        .create_autocomplete_response(&ctx.http, |f| {
            for mission in missions {
                f.add_string_choice(&mission.id, &mission.id);
            }
            f
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
}

async fn upcoming(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _options: &[CommandDataOption],
) {
    let mut interaction = Interaction::new(ctx, command);
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
                        "**{}**\n<t:{}:F> - <t:{}:R>\n*{}*\n\n",
                        data.name,
                        mission.start.unix_timestamp(),
                        mission.start.unix_timestamp(),
                        data.summary,
                    ));
                }
            }
            interaction.reply(content).await;
        }
        Ok(_) => {
            interaction.reply("Failed to fetch upcoming missions").await;
        }
        Err(e) => {
            error!("failed to fetch upcoming missions: {}", e);
            interaction
                .reply(format!("Failed to fetch upcoming missions: {e}"))
                .await;
        }
    }
}

#[allow(clippy::too_many_lines)]
pub async fn remove(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _options: &[CommandDataOption],
) {
    let mut interaction = Interaction::new(ctx, command);
    super::requires_role(
        MISSION_REVIEWER,
        &command.member.as_ref().unwrap().roles,
        &mut interaction,
    )
    .await;
    let time_format = format_description::parse(TIME_FORMAT).unwrap();
    debug!("fetching missions");
    interaction.reply("Fetching missions...").await;
    let Ok(((Response::UpcomingSchedule(Ok(missions)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await
    else {
        interaction.reply("Failed to fetch missions").await;
        return
    };
    let Some(mission_id) = interaction.choice("Select Mission", &missions.iter().map(|m| (format!(
        "{} - {}",
        m.mission,
        &m
            .start
            .to_timezone(NEW_YORK)
            .format(&time_format)
            .unwrap()
    ), m.id)).collect()).await else {
        interaction.reply("Cancelled").await;
        return
    };
    let mission_id = mission_id.parse().unwrap();
    let scheduled = missions.iter().find(|m| m.id == mission_id).unwrap();
    interaction
        .reply(format!(
            "{} - {}",
            scheduled.mission,
            &scheduled
                .start
                .to_timezone(NEW_YORK)
                .format(&time_format)
                .unwrap()
        ))
        .await;
    let confirm = interaction
        .confirm(&format!(
            "Are you sure you want to remove `{} - {}`?",
            scheduled.mission,
            &scheduled
                .start
                .to_timezone(NEW_YORK)
                .format(&time_format)
                .unwrap()
        ))
        .await;
    match confirm {
        Confirmation::Yes => {
            if let Ok(((Response::Unschedule(Ok(())), _), _)) = events_request!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                Unschedule {
                    scheduled_mission: mission_id
                }
            )
            .await
            {
                if let Some(mid) = &scheduled.schedule_message_id {
                    if let Err(e) = SCHEDULE
                        .delete_message(&ctx, MessageId(mid.parse().unwrap()))
                        .await
                    {
                        error!("failed to delete schedule message: {}", e);
                    }
                }
                interaction
                    .reply(format!(
                        "Removed `{} - {}`",
                        scheduled.mission,
                        &scheduled
                            .start
                            .to_timezone(NEW_YORK)
                            .format(&time_format)
                            .unwrap()
                    ))
                    .await;
            } else {
                interaction
                    .reply(format!(
                        "Failed to remove `{} - {}`",
                        scheduled.mission,
                        &scheduled
                            .start
                            .to_timezone(NEW_YORK)
                            .format(&time_format)
                            .unwrap()
                    ))
                    .await;
            }
        }
        Confirmation::No => {
            interaction.reply("Cancelled mission removal").await;
        }
        Confirmation::Timeout => {}
    }
}

#[allow(clippy::too_many_lines)]
async fn post(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _options: &[CommandDataOption],
) {
    let mut interaction = Interaction::new(ctx, command);
    super::requires_role(
        MISSION_REVIEWER,
        &command.member.as_ref().unwrap().roles,
        &mut interaction,
    )
    .await;
    debug!("fetching missions");
    interaction.reply("Fetching missions...").await;
    let Ok(((Response::UpcomingSchedule(Ok(missions)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await
    else {
        interaction.reply("Failed to fetch missions").await;
        return
    };
    let next_unposted = missions.iter().find(|m| m.schedule_message_id.is_none());
    let Some(mission) = next_unposted else {
        interaction.reply("No unposted missions").await;
        return
    };
    debug!("sending confirmation");
    let confirm = interaction
        .confirm(&format!(
            "Are you sure you want to post `{} - {}`?",
            mission.mission,
            mission
                .start
                .to_timezone(NEW_YORK)
                .format(&format_description::parse(TIME_FORMAT).unwrap())
                .unwrap()
        ))
        .await;
    match confirm {
        Confirmation::Yes => {
            if let Ok(((Response::FetchMission(Ok(Some(mission_data))), _), _)) = events_request!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                FetchMission {
                    mission: mission.mission.clone()
                }
            )
            .await
            {
                let Ok(((Response::FetchMissionRsvps(Ok(rsvps)), _), _)) =
                    events_request!(
                        bootstrap::NC::get().await,
                        synixe_events::missions::db,
                        FetchMissionRsvps { mission: mission_data.id.clone() }
                    )
                    .await
                else {
                    return interaction.reply("Failed to fetch rsvps").await;
                };
                // let sched = SCHEDULE
                let sched = BOT
                    .send_message(&ctx, |s| {
                        s.embed(|f| {
                            make_post_embed(f, &mission_data, mission, &rsvps);
                            f
                        });
                        s.components(|c| {
                            c.create_action_row(|r| {
                                r.create_button(|b| {
                                    b.style(ButtonStyle::Primary)
                                        .custom_id("yes")
                                        .emoji(ReactionType::Unicode("🟩".to_string()))
                                })
                                .create_button(|b| {
                                    b.style(ButtonStyle::Primary)
                                        .custom_id("maybe")
                                        .emoji(ReactionType::Unicode("🟨".to_string()))
                                })
                                .create_button(|b| {
                                    b.style(ButtonStyle::Primary)
                                        .custom_id("no")
                                        .emoji(ReactionType::Unicode("🟥".to_string()))
                                })
                            })
                        })
                    })
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
                let sched_thread = SCHEDULE
                    .create_public_thread(&ctx, sched.id, |t| t.name(&mission_data.name))
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
                sched_thread
                    .send_message(&ctx, |pt| {
                        pt.content(
                            mission_data
                                .description
                                .replace("            <br/>", "\n")
                                .replace("<font color='#D81717'>", "")
                                .replace("<font color='#1D69F6'>", "")
                                .replace("<font color='#993399'>", "")
                                .replace("<font color='#663300'>", "")
                                .replace("</font color>", "") // felix you scoundrel
                                .replace("</font>", ""),
                        )
                    })
                    .await
                    .unwrap();
                if let Ok(((Response::SetScheduledMesssage(Ok(())), _), _)) = events_request!(
                    bootstrap::NC::get().await,
                    synixe_events::missions::db,
                    SetScheduledMesssage {
                        scheduled_mission: mission.id,
                        schedule_message_id: sched.id.0.to_string(),
                    }
                )
                .await
                {
                    interaction
                        .reply(format!("Posted `{}`", mission.mission))
                        .await;
                } else {
                    interaction
                        .reply(format!("Failed to post `{}`", mission.mission))
                        .await;
                }
            }
        }
        Confirmation::No => {
            interaction.reply("Cancelled mission posting").await;
        }
        Confirmation::Timeout => {}
    }
}

fn make_post_embed(
    embed: &mut CreateEmbed,
    mission: &Mission,
    schedule: &ScheduledMission,
    rsvps: &[MissionRsvp],
) {
    let mut yes = Vec::new();
    let mut maybe = Vec::new();
    let mut no = Vec::new();
    for rsvp in rsvps {
        match rsvp.state {
            Rsvp::Yes => yes.push(rsvp),
            Rsvp::Maybe => maybe.push(rsvp),
            Rsvp::No => no.push(rsvp),
        }
    }

    embed.title(&mission.name);
    embed.description(&mission.description);
    embed.color(0x00ff_d731);
    embed.timestamp(schedule.start.format(&Rfc3339).unwrap());
    embed.field(
        format!("🟩 Confirmed ({})", yes.len()),
        yes.iter()
            .map(|r| format!("> <@{}>", r.member))
            .collect::<Vec<_>>()
            .join("\n"),
        true,
    );
    embed.field(
        format!("🟨 Maybe ({})", maybe.len()),
        maybe
            .iter()
            .map(|r| format!("> <@{}>", r.member))
            .collect::<Vec<_>>()
            .join("\n"),
        true,
    );
    embed.field(
        format!("🟥 Declined ({})", no.len()),
        no.iter()
            .map(|r| format!("> <@{}>", r.member))
            .collect::<Vec<_>>()
            .join("\n"),
        true,
    );
}
