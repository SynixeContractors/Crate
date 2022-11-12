use std::time::Duration;

use serenity::{
    builder::CreateApplicationCommand,
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        prelude::{
            application_command::CommandDataOption, command::CommandOptionType, MessageId,
            ReactionType,
        },
    },
    prelude::*,
};
use synixe_events::missions::db::{Response, ScheduledFilter};
use synixe_meta::discord::{channel::SCHEDULE, role::MISSION_REVIEWER};
use synixe_proc::events_request;
use time::format_description;
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
                        .name("hour")
                        .description("The starting hour to schedule the mission")
                        .kind(CommandOptionType::Integer)
                        .max_int_value(23)
                        .min_int_value(0)
                        .required(false)
                })
                .create_sub_option(|option| {
                    option
                        .name("unplayed")
                        .description("Only show that haven't been previously scheduled")
                        .kind(CommandOptionType::Boolean)
                        .required(false)
                })
                .create_sub_option(|option| {
                    option
                        .name("search")
                        .description("Filter the list of missions by name")
                        .kind(CommandOptionType::String)
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
    if let Ok(((Response::IsScheduled(Ok(Some(false) | None)), _), _)) = events_request!(
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
    debug!("fetching missions");
    interaction.reply("Fetching missions...").await;
    let filter = options
        .iter()
        .find(|option| option.name == "unplayed")
        .map_or_else(
            || None,
            |option| {
                if option.value.as_ref().unwrap().as_bool().unwrap() {
                    Some(ScheduledFilter::OnlyUnscheduled)
                } else {
                    None
                }
            },
        );
    let search = options
        .iter()
        .find(|option| option.name == "search")
        .map_or_else(
            || None,
            |option| Some(option.value.as_ref().unwrap().as_str().unwrap().to_string()),
        );
    let Ok(((Response::FetchMissionList(Ok(missions)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionList {
            filter,
            search,
        }
    )
    .await else {
        interaction.reply("Failed to fetch missions.").await;
        return;
    };
    let Some(mission_id) = interaction.choice(&format!(
        "Select mission for <t:{}:F>",
        date.unix_timestamp()
    ), &missions.iter().map(|m| (m.id.clone(), m.id.clone())).collect()).await else {
        interaction.reply("No mission selected.").await;
        return;
    };
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
                    .reply(format!("Failed to schedule mission: {}", e))
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
                .reply(format!("Failed to fetch upcoming missions: {}", e))
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
                let sched = SCHEDULE
                    .send_message(&ctx, |s| {
                        s.content(format!(
                            "**{}**\n<t:{}:F> - <t:{}:R>\n\n{}",
                            mission_data.name,
                            mission.start.unix_timestamp(),
                            mission.start.unix_timestamp(),
                            mission_data.summary
                        ))
                    })
                    .await
                    .unwrap();
                for reaction in ["🟩", "🟨", "🟥"] {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    if let Err(e) = sched
                        .react(&ctx, ReactionType::Unicode(reaction.to_string()))
                        .await
                    {
                        error!("Failed to react: {}", e);
                    }
                }
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
