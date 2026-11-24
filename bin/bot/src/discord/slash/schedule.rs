use std::fmt::Write;
use std::time::Duration;

use regex::{Captures, Regex};
use serenity::{
    all::{
        ButtonStyle, ChannelId, ChannelType, CommandData, CommandDataOption,
        CommandDataOptionValue, CommandInteraction, CommandOptionType, ComponentInteraction,
        Message, ReactionType, UserId,
    },
    builder::{
        CreateActionRow, CreateAutocompleteResponse, CreateButton, CreateCommand,
        CreateCommandOption, CreateEmbed, CreateInteractionResponse,
        CreateInteractionResponseMessage, CreateMessage, CreateThread, EditMessage,
    },
    prelude::*,
};
use synixe_events::missions::db::Response;
use synixe_meta::discord::{
    GUILD,
    channel::{LOBBY, LOG, LOOKING_TO_PLAY, ONBOARDING, SCHEDULE},
    role::{FTL, MEMBER, MISSION_REVIEWER, STAFF},
};
use synixe_model::missions::{MissionRsvp, Rsvp, ScheduledMission};
use synixe_proc::events_request_5;
use time::format_description;
use time_tz::{OffsetDateTimeExt, timezones::db::america::NEW_YORK};

use crate::{
    discord::interaction::{Confirmation, Interaction},
    get_option,
};

use super::{AllowPublic, ShouldAsk};

const TIME_FORMAT: &str =
    "[year]-[month]-[day] [hour]:[minute] [offset_hour sign:mandatory]:[offset_minute]";

#[allow(clippy::too_many_lines)]
pub fn register() -> CreateCommand {
    CreateCommand::new("schedule")
        .description("Contract Schedule")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "new",
                "Add a mission to the schedule",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "mission",
                    "Mission to schedule",
                )
                .required(true)
                .set_autocomplete(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "month",
                    "The month to schedule the mission",
                )
                .max_int_value(12)
                .min_int_value(1)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "day",
                    "The day to schedule the mission",
                )
                .max_int_value(31)
                .min_int_value(1)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "hour",
                    "The starting hour to schedule the mission",
                )
                .max_int_value(23)
                .min_int_value(0)
                .required(false),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "subcon",
                "Propose a subcon mission",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "month",
                    "The month to schedule the mission",
                )
                .max_int_value(12)
                .min_int_value(1)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "day",
                    "The day to schedule the mission",
                )
                .max_int_value(31)
                .min_int_value(1)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "hour",
                    "The starting hour to schedule the mission",
                )
                .max_int_value(23)
                .min_int_value(0)
                .required(false),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "Channel to post the upcoming mission",
                )
                .required(false),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "upcoming",
                "View the upcoming missions",
            )
            .allow_public(),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "remove",
            "Remove an upcoming mission",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "lead",
                "Claim, release or assign an upcoming mission's Operation Lead",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "action", "What to do")
                    .add_string_choice("Claim it", "claim")
                    .add_string_choice("Release it", "release")
                    .add_string_choice("Assign someone (staff)", "assign")
                    .required(true),
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::User,
                "member",
                "Who to assign, for assign",
            )),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "post",
                "Post the upcoming mission",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "Channel to post the upcoming mission",
                )
                .required(false),
            ),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for bank provided");
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(values) = &subcommand.value {
        match subcommand.name.as_str() {
            "new" => new(ctx, command, values).await?,
            "subcon" => subcon(ctx, command, values).await?,
            "upcoming" => upcoming(ctx, command, values).await?,
            "remove" => remove(ctx, command, values).await?,
            "lead" => lead(ctx, command, values).await?,
            "post" => post(ctx, command, values).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub async fn autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
) -> serenity::Result<()> {
    let Some(subcommand) = autocomplete.data.options.first() else {
        warn!("No subcommand for bank provided");
        return Ok(());
    };
    if subcommand.kind() == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "new" => new_autocomplete(ctx, autocomplete).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::too_many_lines)]
pub async fn rsvp_button(ctx: &Context, component: &ComponentInteraction) -> serenity::Result<()> {
    let channel = component.channel_id;
    let message = component.message.id;
    let Ok(Ok((Response::FetchScheduledMessage(Ok(Some(scheduled))), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchScheduledMessage { channel, message }
    )
    .await
    else {
        error!("Failed to fetch scheduled mission for component");
        return Ok(());
    };
    let Ok(member) = GUILD.member(&ctx, component.user.id).await else {
        error!("Failed to fetch member for component");
        return Ok(());
    };
    if member.roles.is_empty() {
        if let Err(e) = component
            .create_response(&ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::default()
                    .content(format!(
                        "You must be a member to RSVP. Head to <#{LOBBY}> and ask any questions you may have, or checkout <#{ONBOARDING}> to apply and get started!")
                    )
                    .ephemeral(true)
            ))
            .await
        {
            error!("Failed to create interaction response: {}", e);
        }
        return Ok(());
    }
    let before = member_rsvp(scheduled.id, component.user.id)
        .await
        .map(|rsvp| rsvp.state);
    match component.data.custom_id.as_str() {
        "rsvp_yes" => {
            let Ok(Ok((Response::AddMissionRsvp(Ok(())), _))) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                AddMissionRsvp {
                    scheduled: scheduled.id,
                    member: component.user.id.to_string(),
                    rsvp: Rsvp::Yes,
                    details: None,
                }
            )
            .await
            else {
                error!("Failed to add mission rsvp for component");
                return Ok(());
            };
            if let Err(e) = component
                .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
                .await
            {
                error!("Failed to create interaction response: {}", e);
            }
        }
        "rsvp_maybe" => {
            let mut interaction = Interaction::new(ctx, component.clone(), &[]);
            let Some(reason) = interaction
                .choice("Please provide a reason, this helps us make informed decision to improve Synixe!",
                &[("I might not be able to make it".to_string(), "not_sure".to_string()),
                    ("I'm not interested in this mission".to_string(), "not_interested".to_string()),
                    ("I'm burnt out and may not want to attend".to_string(), "burnt_out".to_string()),
                    ("Other".to_string(), "other".to_string())]
                )
                .await?
            else {
                warn!("No reason provided for rsvp_maybe");
                return Ok(());
            };
            let Ok(Ok((Response::AddMissionRsvp(Ok(())), _))) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                AddMissionRsvp {
                    scheduled: scheduled.id,
                    member: component.user.id.to_string(),
                    rsvp: Rsvp::Maybe,
                    details: Some(reason),
                }
            )
            .await
            else {
                error!("Failed to add mission rsvp for component");
                return Ok(());
            };
            interaction.reply("Thank you for your RSVP!").await?;
        }
        "rsvp_no" => {
            let mut interaction = Interaction::new(ctx, component.clone(), &[]);
            let Some(reason) = interaction
                .choice("Please provide a reason, this helps us make informed decision to improve Synixe!",
                    &[("I won't be able to make it".to_string(), "not_sure".to_string()),
                        ("I'm not interested in this mission".to_string(), "not_interested".to_string()),
                        ("I'm burnt out".to_string(), "burnt_out".to_string()),
                        ("Other".to_string(), "other".to_string())]
                )
                .await?
            else {
                warn!("No reason provided for rsvp_no");
                return Ok(());
            };
            let Ok(Ok((Response::AddMissionRsvp(Ok(())), _))) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                AddMissionRsvp {
                    scheduled: scheduled.id,
                    member: component.user.id.to_string(),
                    rsvp: Rsvp::No,
                    details: Some(reason),
                }
            )
            .await
            else {
                error!("Failed to add mission rsvp for component");
                return Ok(());
            };
            interaction.reply("Thank you for your RSVP!").await?;
        }
        _ => {
            warn!("Unknown component id: {}", component.data.custom_id);
        }
    }
    let Ok(Ok((Response::FetchMissionRsvps(Ok(rsvps)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionRsvps {
            scheduled: scheduled.id
        }
    )
    .await
    else {
        return Ok(());
    };
    if let Some(after) = rsvps
        .iter()
        .find(|rsvp| rsvp.member == component.user.id.to_string())
    {
        log_late_change(ctx, &scheduled, component.user.id, before, after).await;
    }
    if let Err(e) = component
        .channel_id
        .edit_message(
            &ctx.http,
            message,
            EditMessage::default()
                .embed(make_post_embed(ctx, &scheduled, &rsvps).await)
                .components(vec![rsvp_buttons()]),
        )
        .await
    {
        error!("Failed to edit message: {}", e);
    }
    Ok(())
}

/// RSVP changes this close to the start are reported, so the operation lead can plan around them
const LATE_CHANGE: time::Duration = time::Duration::hours(3);

async fn member_rsvp(scheduled: uuid::Uuid, member: UserId) -> Option<MissionRsvp> {
    let Ok(Ok((Response::FetchMissionRsvps(Ok(rsvps)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionRsvps { scheduled }
    )
    .await
    else {
        return None;
    };
    rsvps
        .into_iter()
        .find(|rsvp| rsvp.member == member.to_string())
}

const fn rsvp_label(rsvp: Rsvp) -> &'static str {
    match rsvp {
        Rsvp::Yes => "🟩 Attending",
        Rsvp::Maybe => "🟨 Maybe",
        Rsvp::No => "🟥 Declined",
    }
}

/// Posts in #log when a member changes their RSVP within `LATE_CHANGE` of the mission starting
async fn log_late_change(
    ctx: &Context,
    scheduled: &ScheduledMission,
    member: UserId,
    before: Option<Rsvp>,
    after: &MissionRsvp,
) {
    if before == Some(after.state)
        || scheduled.start - time::OffsetDateTime::now_utc() > LATE_CHANGE
    {
        return;
    }
    let reason = after
        .details
        .as_deref()
        .map_or_else(String::new, |details| {
            format!("\n> {}", details.replace('_', " "))
        });
    if let Err(e) = LOG
        .say(
            &ctx.http,
            format!(
                "<@{member}> changed their RSVP for **{}** (starts <t:{}:R>): {} → {}{reason}",
                scheduled.name,
                scheduled.start.unix_timestamp(),
                before.map_or("No RSVP", rsvp_label),
                rsvp_label(after.state),
            ),
        )
        .await
    {
        error!("Failed to log late rsvp change: {}", e);
    }
}

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::too_many_lines)]
async fn new(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    let date = super::get_datetime(options);
    super::requires_roles(
        command.user.id,
        &[MISSION_REVIEWER, STAFF],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Deny,
        &mut interaction,
    )
    .await?;
    if let Ok(Ok((Response::IsScheduled(Ok(Some(Some(false) | None) | None)), _))) =
        events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            IsScheduled { date }
        )
        .await
    {
        debug!("No mission scheduled for {}", date);
    } else {
        return interaction
            .reply(format!(
                "A mission is already scheduled at <t:{}:F>, or the check failed.",
                date.unix_timestamp()
            ))
            .await;
    }
    let Some(mission_id) = get_option!(options, "mission", String) else {
        return interaction
            .reply("Required option not provided: mission")
            .await;
    };
    let Ok(Ok((Response::FetchMissionList(Ok(missions)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionList {
            search: Some(mission_id.clone()),
        }
    )
    .await
    else {
        error!("failed to fetch mission list");
        return Ok(());
    };
    if missions.len() != 1 {
        return interaction
            .reply(format!(
                "Found {} missions matching `{}`",
                missions.len(),
                mission_id
            ))
            .await;
    }
    let confirm = interaction
        .confirm(&format!(
            "Schedule `{}` for <t:{}:F>?",
            mission_id,
            date.unix_timestamp()
        ))
        .await?;
    match confirm {
        Confirmation::Yes => {
            if let Err(e) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                Schedule {
                    mission: mission_id.clone(),
                    date
                }
            )
            .await
            {
                error!("failed to schedule mission: {}", e);
                interaction
                    .reply(format!("Failed to schedule mission: {e}"))
                    .await?;
            } else {
                interaction
                    .reply(format!(
                        "Scheduled `{}` for <t:{}:F>",
                        mission_id,
                        date.unix_timestamp()
                    ))
                    .await?;
            }
        }
        Confirmation::No => {
            interaction.reply("Cancelled.").await?;
        }
        Confirmation::Timeout => {}
    }
    Ok(())
}

async fn new_autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
) -> serenity::Result<()> {
    let Some(focus) = CommandData::autocomplete(&autocomplete.data) else {
        return Ok(());
    };
    if focus.name != "mission" {
        return Ok(());
    }
    let Ok(Ok((Response::FetchMissionList(Ok(mut missions)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionList {
            search: Some(focus.value.to_string())
        }
    )
    .await
    else {
        error!("failed to fetch mission list");
        return Ok(());
    };
    // Show unplayed when no search term is provided
    if focus.value.is_empty() {
        missions.retain(|m| {
            m.play_count.unwrap_or_default() == 0
                && !m.id.to_lowercase().starts_with("tt30")
                && !m.id.to_lowercase().starts_with("tra30")
        });
    }
    missions.truncate(25);
    if let Err(e) = autocomplete
        .create_response(&ctx.http, {
            let mut f = CreateAutocompleteResponse::default();
            for mission in missions {
                f = f.add_string_choice(&mission.id, &mission.id);
            }
            CreateInteractionResponse::Autocomplete(f)
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn subcon(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    let date = super::get_datetime(options);
    super::requires_roles(
        command.user.id,
        &[MEMBER, STAFF],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Yes(("mission subcon", options)),
        &mut interaction,
    )
    .await?;
    let Ok(Ok((Response::IsScheduled(Ok(Some(Some(false) | None) | None)), _))) =
        events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            IsScheduled { date }
        )
        .await
    else {
        return interaction
            .reply(format!(
                "A mission is already scheduled at <t:{}:F>, or the check failed.",
                date.unix_timestamp()
            ))
            .await;
    };
    interaction.reply("Scheduling mission...").await?;
    let Ok(Ok((Response::Schedule(Ok(Some(scheduled))), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        Schedule {
            mission: "$SUBCON$".to_string(),
            date
        }
    )
    .await
    else {
        error!("failed to schedule mission");
        return interaction.reply("Failed to schedule mission").await;
    };
    let channel = get_option!(options, "channel", Channel).map_or_else(|| LOOKING_TO_PLAY, |c| *c);
    let Ok(sched) = channel
        .send_message(
            &ctx,
            CreateMessage::default()
                .embed(make_post_embed(ctx, &scheduled, &[]).await)
                .components(vec![rsvp_buttons()]),
        )
        .await
    else {
        return interaction.reply("Failed to post mission").await;
    };
    if let Ok(Ok((Response::SetScheduledMesssage(Ok(())), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        SetScheduledMesssage {
            scheduled: scheduled.id,
            channel,
            message: sched.id,
        }
    )
    .await
    {
        interaction
            .reply(format!(
                "A subcon is proposed for <t:{}:F>",
                date.unix_timestamp()
            ))
            .await?;
    } else {
        interaction
            .reply("Failed to post subcon, please contact a staff member.")
            .await?;
    }
    Ok(())
}

async fn upcoming(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await
    {
        Ok(Ok((Response::UpcomingSchedule(Ok(upcoming)), _))) => {
            let mut content = String::from("**Upcoming Missions**\n\n");
            for scheduled in upcoming {
                write!(
                    content,
                    "**{}**\n<t:{}:F> - <t:{}:R>\n*{}*\n\n",
                    scheduled.mission,
                    scheduled.start.unix_timestamp(),
                    scheduled.start.unix_timestamp(),
                    scheduled.summary,
                )?;
            }
            interaction.reply(content).await?;
        }
        Ok(_) => {
            interaction
                .reply("Failed to fetch upcoming missions")
                .await?;
        }
        Err(e) => {
            error!("failed to fetch upcoming missions: {}", e);
            interaction
                .reply(format!("Failed to fetch upcoming missions: {e}"))
                .await?;
        }
    }
    Ok(())
}

/// Claims, releases or assigns an upcoming mission's lead. FTLs can claim an unclaimed lead, and anyone
/// leading can release their own. Staff can do all of it, and release or assign any lead.
#[allow(clippy::too_many_lines)]
async fn lead(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    let Some(action) = get_option!(options, "action", String) else {
        return interaction.reply("Invalid action").await;
    };
    let member = command.user.id;
    let roles = &command
        .member
        .as_ref()
        .expect("member should always exist on guild commands")
        .roles;
    let staff = roles.contains(&STAFF);
    let assignee = match action.as_str() {
        "claim" if !staff && !roles.contains(&FTL) => {
            return interaction
                .reply(format!(
                    "Only <@&{FTL}> and staff can claim the Operation Lead"
                ))
                .await;
        }
        "assign" if !staff => {
            return interaction
                .reply("Only staff can assign an Operation Lead")
                .await;
        }
        "assign" => {
            let Some(assignee) = get_option!(options, "member", User) else {
                return interaction.reply("Pick the member to assign").await;
            };
            Some(*assignee)
        }
        _ => None,
    };

    interaction.reply("Fetching missions...").await?;
    let Ok(Ok((Response::UpcomingSchedule(Ok(mut missions)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await
    else {
        return interaction.reply("Failed to fetch missions").await;
    };
    if missions.is_empty() {
        return interaction.reply("There are no upcoming missions").await;
    }
    missions.sort_by_key(|m| m.start);
    let time_format = format_description::parse_borrowed::<3>(TIME_FORMAT)
        .expect("Time format should have been valid");
    let Some(scheduled) = interaction
        .choice(
            "Select Mission",
            &missions
                .iter()
                .map(|m| {
                    (
                        format!(
                            "{} - {}",
                            m.name,
                            m.start
                                .to_timezone(NEW_YORK)
                                .format(&time_format)
                                .unwrap_or_default()
                        ),
                        m.id,
                    )
                })
                .collect::<Vec<_>>(),
        )
        .await?
    else {
        return interaction.reply("Cancelled").await;
    };
    let Some(scheduled) = uuid::Uuid::parse_str(&scheduled)
        .ok()
        .and_then(|id| missions.iter().find(|m| m.id == id))
    else {
        return interaction.reply("Mission not found").await;
    };
    let name = &scheduled.name;

    let reply = match action.as_str() {
        "claim" => match events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            ClaimOperationLead {
                scheduled: scheduled.id,
                member
            }
        )
        .await
        {
            Ok(Ok((Response::ClaimOperationLead(Ok(Some(lead))), _))) if lead == member => {
                format!("You're leading **{name}**")
            }
            Ok(Ok((Response::ClaimOperationLead(Ok(Some(lead))), _))) => {
                return interaction
                    .reply(format!(
                        "<@{lead}> is already leading **{name}**{}",
                        if staff {
                            ". Release it first, or assign someone."
                        } else {
                            ". They can release it if plans change."
                        }
                    ))
                    .await;
            }
            _ => return interaction.reply("Failed to claim the lead").await,
        },
        "release" => {
            if !release_lead(scheduled.id, member, staff).await {
                return interaction.reply("Failed to release the lead").await;
            }
            match events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                FetchOperationLead {
                    scheduled: scheduled.id
                }
            )
            .await
            {
                Ok(Ok((Response::FetchOperationLead(Ok(Some(lead))), _))) => {
                    return interaction
                        .reply(format!(
                            "Only <@{lead}>, who's leading **{name}**, can release it"
                        ))
                        .await;
                }
                _ => format!("**{name}** has no Operation Lead now"),
            }
        }
        "assign" => {
            let Ok(Ok((Response::SetOperationLead(Ok(())), _))) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                SetOperationLead {
                    scheduled: scheduled.id,
                    member: assignee
                }
            )
            .await
            else {
                return interaction.reply("Failed to assign the lead").await;
            };
            format!(
                "<@{}> is leading **{name}**",
                assignee.expect("assign always has an assignee")
            )
        }
        _ => return interaction.reply("Invalid action").await,
    };
    refresh_post(ctx, scheduled.id).await;
    interaction.reply(reply).await
}

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::too_many_lines)]
pub async fn remove(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    super::requires_roles(
        command.user.id,
        &[MISSION_REVIEWER, STAFF],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Deny,
        &mut interaction,
    )
    .await?;
    let time_format = format_description::parse_borrowed::<3>(TIME_FORMAT)
        .expect("Time format should have been valid");
    debug!("fetching missions");
    interaction.reply("Fetching missions...").await?;
    let Ok(Ok((Response::UpcomingSchedule(Ok(missions)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await
    else {
        return interaction.reply("Failed to fetch missions").await;
    };
    let Some(scheduled_id) = interaction
        .choice(
            "Select Mission",
            &missions
                .iter()
                .map(|m| {
                    (
                        format!(
                            "{} - {}",
                            m.mission,
                            m.start
                                .to_timezone(NEW_YORK)
                                .format(&time_format)
                                .expect("Should have been able to format time")
                        ),
                        m.id,
                    )
                })
                .collect::<Vec<_>>(),
        )
        .await?
    else {
        return interaction.reply("Cancelled").await;
    };
    let scheduled_id = scheduled_id.parse().expect("Should have been a valid uuid");
    let scheduled = missions
        .iter()
        .find(|m| m.id == scheduled_id)
        .expect("Options are limited to this list");
    interaction
        .reply(format!(
            "{} - {}",
            scheduled.mission,
            scheduled
                .start
                .to_timezone(NEW_YORK)
                .format(&time_format)
                .expect("Should have been able to format time")
        ))
        .await?;
    let confirm = interaction
        .confirm(&format!(
            "Are you sure you want to remove `{} - {}`?",
            scheduled.mission,
            scheduled
                .start
                .to_timezone(NEW_YORK)
                .format(&time_format)
                .expect("Should have been able to format time")
        ))
        .await?;
    match confirm {
        Confirmation::Yes => {
            if let Ok(Ok((Response::Unschedule(Ok(())), _))) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                Unschedule {
                    scheduled: scheduled_id
                }
            )
            .await
            {
                if let Some((channel, message)) = &scheduled.message()
                    && let Err(e) = channel.delete_message(&ctx, message).await
                {
                    error!("failed to delete schedule message: {}", e);
                }
                interaction
                    .reply(format!(
                        "Removed `{} - {}`",
                        scheduled.mission,
                        scheduled
                            .start
                            .to_timezone(NEW_YORK)
                            .format(&time_format)
                            .expect("Should have been able to format time")
                    ))
                    .await?;
            } else {
                interaction
                    .reply(format!(
                        "Failed to remove `{} - {}`",
                        scheduled.mission,
                        scheduled
                            .start
                            .to_timezone(NEW_YORK)
                            .format(&time_format)
                            .expect("Should have been able to format time")
                    ))
                    .await?;
            }
        }
        Confirmation::No => {
            interaction.reply("Cancelled mission removal").await?;
        }
        Confirmation::Timeout => {}
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn post(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    super::requires_roles(
        command.user.id,
        &[MISSION_REVIEWER, STAFF],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Deny,
        &mut interaction,
    )
    .await?;
    debug!("fetching missions");
    interaction.reply("Fetching missions...").await?;
    let Ok(Ok((Response::UpcomingSchedule(Ok(missions)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await
    else {
        return interaction.reply("Failed to fetch missions").await;
    };
    let next_unposted = missions.iter().find(|m| m.schedule_message_id.is_none());
    let Some(scheduled) = next_unposted else {
        return interaction.reply("No unposted missions").await;
    };
    debug!("sending confirmation");
    let confirm = interaction
        .confirm(&format!(
            "Are you sure you want to post `{} - {}`?",
            scheduled.mission,
            scheduled
                .start
                .to_timezone(NEW_YORK)
                .format(
                    &format_description::parse_borrowed::<3>(TIME_FORMAT)
                        .expect("Time format should have been valid")
                )
                .expect("Should have been able to format time")
        ))
        .await?;
    match confirm {
        Confirmation::Yes => {
            let channel = get_option!(options, "channel", Channel).map_or_else(|| SCHEDULE, |c| *c);
            post_mission(ctx, channel, scheduled, Some(interaction), true).await;
        }
        Confirmation::No => {
            interaction.reply("Cancelled mission posting").await?;
        }
        Confirmation::Timeout => {}
    }
    Ok(())
}

async fn make_post_embed(
    ctx: &Context,
    scheduled: &ScheduledMission,
    rsvps: &[MissionRsvp],
) -> CreateEmbed {
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

    let lead = match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchOperationLead {
            scheduled: scheduled.id
        }
    )
    .await
    {
        Ok(Ok((Response::FetchOperationLead(Ok(Some(lead))), _))) => format!("<@{lead}>"),
        _ => String::from("Not decided yet"),
    };
    CreateEmbed::default()
        .title(&scheduled.name)
        .description(&scheduled.summary)
        .color(0x00ff_d731)
        .field(
            "🕒 Time",
            format!(
                "<t:{}:F> - <t:{}:R>",
                scheduled.start.unix_timestamp(),
                scheduled.start.unix_timestamp()
            ),
            false,
        )
        .field("🎖️ Operation Lead", lead, false)
        .field(
            format!("🟩 Attending ({})", yes.len()),
            {
                let out = alphabetical(ctx, &yes)
                    .await
                    .iter()
                    .map(|r| format!("> <@{}>", r.member))
                    .collect::<Vec<_>>()
                    .join("\n");
                if out.is_empty() { "-".to_string() } else { out }
            },
            true,
        )
        .field(
            format!("🟨 Maybe ({})", maybe.len()),
            {
                let out = alphabetical(ctx, &maybe)
                    .await
                    .iter()
                    .map(|r| format!("> <@{}>", r.member))
                    .collect::<Vec<_>>()
                    .join("\n");
                if out.is_empty() { "-".to_string() } else { out }
            },
            true,
        )
        .field(
            format!("🟥 Declined ({})", no.len()),
            {
                let out = alphabetical(ctx, &no)
                    .await
                    .iter()
                    .map(|r| format!("> <@{}>", r.member))
                    .collect::<Vec<_>>()
                    .join("\n");
                if out.is_empty() { "-".to_string() } else { out }
            },
            true,
        )
}

async fn alphabetical<'a>(ctx: &Context, rsvps: &'a [&'a MissionRsvp]) -> Vec<&'a MissionRsvp> {
    let mut ret_rsvps = Vec::new();
    for rsvp in rsvps {
        if let Err(e) = GUILD
            .member(ctx, rsvp.member.parse::<u64>().unwrap_or(0))
            .await
            .map(|m| {
                ret_rsvps.push((m.display_name().to_string(), *rsvp));
            })
        {
            error!("Failed to fetch member for rsvp: {} - {}", rsvp.member, e);
        }
    }
    ret_rsvps.sort_by(|a, b| a.0.cmp(&b.0));
    ret_rsvps.into_iter().map(|(_, rsvp)| rsvp).collect()
}

fn rsvp_buttons() -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        CreateButton::new("rsvp_yes")
            .style(ButtonStyle::Secondary)
            .emoji(ReactionType::Unicode("🟩".to_string())),
        CreateButton::new("rsvp_maybe")
            .style(ButtonStyle::Secondary)
            .emoji(ReactionType::Unicode("🟨".to_string())),
        CreateButton::new("rsvp_no")
            .style(ButtonStyle::Secondary)
            .emoji(ReactionType::Unicode("🟥".to_string())),
    ])
}

fn briefing_messages(mission: &ScheduledMission) -> Vec<String> {
    let mut text = String::new();
    let briefing = mission.briefing();
    for title in ["Situation", "Mission", "Objectives"] {
        let Some(content) = briefing.get(&title.to_lowercase()) else {
            continue;
        };
        writeln!(text, "## {title}").expect("valid format");
        text.push_str(content);
    }

    let img_regex = Regex::new(r"(?m)<img ([^>]+?)\/>").expect("valid regex");
    let img_path_regex = Regex::new(r#"(?m)image=(?:'|")([^'"]+)"#).expect("valid regex");
    let img_name_regex = Regex::new(r#"(?m)name=(?:'|")([^'"]+)"#).expect("valid regex");

    let out = img_regex.replace_all(&text, |caps: &Captures| {
        let img = caps.get(1).expect("group 1 will always exist").as_str();
        let path = img_path_regex
            .captures(img)
            .expect("img image will always exist")
            .get(1)
            .expect("img image will always exist")
            .as_str();
        let name = {
            let name = img_name_regex
                .captures(img)
                .map(|c| c.get(1).expect("always in capture").as_str());
            name.map_or("image", |name| name)
        };
        format!(
            "[{}](https://raw.githubusercontent.com/SynixeContractors/Missions/main/{}/{}/{})",
            name,
            mission.typ.github_folder().to_lowercase(),
            mission.mission.to_lowercase(),
            path
        )
    });

    let tag_regex = Regex::new(r"(?m)<([^>]+)>").expect("valid regex");
    let out = tag_regex.replace_all(&out, |caps: &Captures| {
        let tag = caps.get(1).expect("group 1 will always exist").as_str();
        if tag.contains("color") {
            "### ".to_string()
        } else {
            String::new()
        }
    });

    let mut sections = Vec::with_capacity(10);
    let mut section = String::with_capacity(2048);
    for line in out.lines() {
        if line.starts_with("## ") {
            if !section.is_empty() {
                sections.push(section.trim_end_matches('\n').to_string());
            }
            section = line.to_string();
        } else {
            section.push('\n');
            section.push_str(line);
        }
    }
    if !section.is_empty() {
        sections.push(section.trim_end_matches('\n').to_string());
    }
    sections
}

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::too_many_lines)]
pub async fn post_mission<'a>(
    ctx: &'a Context,
    channel: ChannelId,
    scheduled: &ScheduledMission,
    interaction: Option<Interaction<'a>>,
    thread: bool,
) -> Option<Message> {
    let Ok(Ok((Response::FetchMissionRsvps(Ok(rsvps)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionRsvps {
            scheduled: scheduled.id
        }
    )
    .await
    else {
        if let Some(mut interaction) = interaction
            && let Err(e) = interaction.reply("Failed to fetch rsvps").await
        {
            error!("Failed to fetch rsvps: {}", e);
        }
        return None;
    };
    let Ok(sched) = channel
        .send_message(
            &ctx,
            CreateMessage::default()
                .embed(make_post_embed(ctx, scheduled, &rsvps).await)
                .components(vec![rsvp_buttons()]),
        )
        .await
    else {
        if let Some(mut interaction) = interaction
            && let Err(e) = interaction.reply("Failed to post mission").await
        {
            error!("Failed to post mission: {}", e);
        }
        return None;
    };
    tokio::time::sleep(Duration::from_millis(500)).await;
    if thread {
        let Ok(sched_thread) = channel
            .create_thread_from_message(
                &ctx,
                sched.id,
                CreateThread::new(&scheduled.name).kind(ChannelType::PublicThread),
            )
            .await
        else {
            if let Some(mut interaction) = interaction
                && let Err(e) = interaction.reply("Failed to create thread").await
            {
                error!("Failed to create thread: {}", e);
            }
            return None;
        };
        tokio::time::sleep(Duration::from_millis(100)).await;
        if let Some(content) = scheduled.briefing().get("old") {
            if let Err(e) = sched_thread
                .say(
                    &ctx,
                    content
                        .replace("            <br/>", "\n")
                        .replace("<font color='#D81717'>", "")
                        .replace("<font color='#1D69F6'>", "")
                        .replace("<font color='#993399'>", "")
                        .replace("<font color='#663300'>", "")
                        .replace("<font color='#139120'>", "")
                        .replace("</font color>", "") // felix you scoundrel
                        .replace("</font>", ""),
                )
                .await
            {
                error!("Failed to post mission description: {}", e);
            }
        } else {
            for section in briefing_messages(scheduled) {
                if let Err(e) = sched_thread
                    .send_message(&ctx, CreateMessage::default().content(section))
                    .await
                {
                    error!("Failed to post mission description: {}", e);
                }
            }
        }
    }
    if let Ok(Ok((Response::SetScheduledMesssage(Ok(())), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        SetScheduledMesssage {
            scheduled: scheduled.id,
            channel,
            message: sched.id,
        }
    )
    .await
    {
        if let Some(mut interaction) = interaction
            && let Err(e) = interaction
                .reply(format!("Posted `{}`", scheduled.mission))
                .await
        {
            error!("Failed to post mission: {}", e);
        }
    } else if let Some(mut interaction) = interaction
        && let Err(e) = interaction
            .reply(format!("Failed to post `{}`", scheduled.mission))
            .await
    {
        error!("Failed to post mission: {}", e);
    }
    Some(sched)
}

/// Prefixes of the operation lead buttons on the 24 hour leadership reminder, followed by the scheduled
/// mission's ID
pub const LEAD_CLAIM: &str = "lead_claim:";
pub const LEAD_RELEASE: &str = "lead_release:";
/// Starts the operation lead line of the leadership reminder, which the buttons rewrite
pub const LEAD_LINE: &str = "\n\n🎖️ Operation Lead: ";

pub fn lead_buttons(scheduled: uuid::Uuid) -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        CreateButton::new(format!("{LEAD_CLAIM}{scheduled}"))
            .style(ButtonStyle::Primary)
            .label("Claim Operation Lead"),
        CreateButton::new(format!("{LEAD_RELEASE}{scheduled}"))
            .style(ButtonStyle::Secondary)
            .label("Release"),
    ])
}

pub fn lead_line(lead: Option<UserId>) -> String {
    lead.map_or_else(
        || format!("{LEAD_LINE}Not claimed yet"),
        |lead| format!("{LEAD_LINE}<@{lead}>"),
    )
}

/// Handles the Claim and Release buttons on the leadership reminder
pub async fn lead_button(ctx: &Context, component: &ComponentInteraction) -> serenity::Result<()> {
    let id = component.data.custom_id.as_str();
    let (claiming, scheduled) = if let Some(rest) = id.strip_prefix(LEAD_CLAIM) {
        (true, rest)
    } else if let Some(rest) = id.strip_prefix(LEAD_RELEASE) {
        (false, rest)
    } else {
        return Ok(());
    };
    let Ok(scheduled) = uuid::Uuid::parse_str(scheduled) else {
        error!("invalid lead button: {id}");
        return Ok(());
    };
    let member = component.user.id;
    let roles = GUILD
        .member(ctx, member)
        .await
        .map_or_else(|_| Vec::new(), |member| member.roles);
    let staff = roles.contains(&STAFF);
    if claiming && !staff && !roles.contains(&FTL) {
        return lead_reply(
            ctx,
            component,
            &format!("Only <@&{FTL}> and staff can claim the Operation Lead"),
        )
        .await;
    }

    let lead = if claiming {
        match events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            ClaimOperationLead { scheduled, member }
        )
        .await
        {
            Ok(Ok((Response::ClaimOperationLead(Ok(lead)), _))) => lead,
            _ => return lead_reply(ctx, component, "Failed to claim the lead, try again").await,
        }
    } else {
        if !release_lead(scheduled, member, staff).await {
            return lead_reply(ctx, component, "Failed to release the lead, try again").await;
        }
        match events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            FetchOperationLead { scheduled }
        )
        .await
        {
            Ok(Ok((Response::FetchOperationLead(Ok(lead)), _))) => lead,
            _ => return lead_reply(ctx, component, "Failed to release the lead, try again").await,
        }
    };

    match lead {
        Some(lead) if claiming && lead != member => {
            return lead_reply(
                ctx,
                component,
                &if staff {
                    format!(
                        "<@{lead}> is already leading this one. Release it first, or use `/schedule lead` to assign someone."
                    )
                } else {
                    format!(
                        "<@{lead}> is already leading this one. They can release it if plans change."
                    )
                },
            )
            .await;
        }
        Some(lead) if !claiming => {
            return lead_reply(
                ctx,
                component,
                &format!("Only <@{lead}>, who's leading it, can release it."),
            )
            .await;
        }
        _ => {}
    }

    let content = component.message.content.as_str();
    let base = content.split(LEAD_LINE).next().unwrap_or(content);
    component
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(format!("{base}{}", lead_line(lead)))
                    .components(vec![lead_buttons(scheduled)]),
            ),
        )
        .await?;
    refresh_post(ctx, scheduled).await;
    Ok(())
}

async fn lead_reply(
    ctx: &Context,
    component: &ComponentInteraction,
    content: &str,
) -> serenity::Result<()> {
    component
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true),
            ),
        )
        .await
}

/// Gives up a mission's lead. Staff release it whoever holds it, anyone else only their own.
async fn release_lead(scheduled: uuid::Uuid, member: UserId, staff: bool) -> bool {
    if staff {
        matches!(
            events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                SetOperationLead {
                    scheduled,
                    member: None
                }
            )
            .await,
            Ok(Ok((Response::SetOperationLead(Ok(())), _)))
        )
    } else {
        matches!(
            events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                ReleaseOperationLead { scheduled, member }
            )
            .await,
            Ok(Ok((Response::ReleaseOperationLead(Ok(())), _)))
        )
    }
}

/// Rebuilds an upcoming mission's #schedule post, e.g. after its operation lead changed
async fn refresh_post(ctx: &Context, scheduled: uuid::Uuid) {
    let Ok(Ok((Response::UpcomingSchedule(Ok(upcoming)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await
    else {
        return;
    };
    let Some(scheduled) = upcoming.into_iter().find(|m| m.id == scheduled) else {
        return;
    };
    let Some((channel, message)) = scheduled.message() else {
        return;
    };
    let Ok(Ok((Response::FetchMissionRsvps(Ok(rsvps)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionRsvps {
            scheduled: scheduled.id
        }
    )
    .await
    else {
        return;
    };
    if let Err(e) = channel
        .edit_message(
            &ctx.http,
            message,
            EditMessage::default().embed(make_post_embed(ctx, &scheduled, &rsvps).await),
        )
        .await
    {
        error!("Failed to refresh the schedule post: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value};
    use synixe_model::missions::MissionType;
    use time::OffsetDateTime;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn briefing() {
        let mission = ScheduledMission {
            id: Uuid::new_v4(),
            mission: "CO30_Brodsky_Test".to_string(),
            schedule_message_id: None,
            start: OffsetDateTime::now_utc(),
            name: "Test Mission".to_string(),
            summary: "Contractors do a test".to_string(),
            briefing: {
                let mut map = Map::new();
                map.insert(
                    "situation".to_string(),
Value::String(r"<font color='#E0A750'>SITUATION</font>
Insert things players should know about where they are deploying and/or events that happened leading up to their deployment or to the given conflict

<font color='#D81717'>ENEMY FORCES</font>
Insert enemies here
Insert enemy AMLCOA (most likely course of action, aka, what they're doing, how they act, how they'll act towards us, what we know, etc)

<font color='#1D69F6'>FRIENDLY FORCES</font>
Insert friendlies here

<font color='#139120'>INDEPENDENT FORCES</font>
Insert independent forces if there are any, otherwise remove this line and one above

<font color='#993399'>CIVILIAN CONSIDERATIONS</font>
Insert things to consider about civilians, presence/absence and/or behaviour/support to friendlies or enemies

<font color='#663300'>TERRAIN CONSIDERATIONS</font>
Insert anything you find relevant about how the terrain may be advantageous or disadvantageous
".to_string())
                );
                map.insert(
                    "mission".to_string(),
                    Value::String(
                        r"<font color='#1D69F6'>EMPLOYER</font>

Insert name of employer here

<font color='#E0A750'>MISSION</font>

Insert your mission description here,

You can add as much as you feel is relevant.

<img image='FOLDER\IMAGE.jpg' width='200' height='100'/>

As a rule of thumb, consider: who, what (tasks), where, when, and why

<font color='#595305'>RESTRICTIONS</font>

Insert any additional restrictions here

If there are none, remove this section
"
                        .to_string(),
                    ),
                );
                map.insert(
                    "objectives".to_string(),
                    Value::String(
                        r"<font color='#E3D310'>PRIMARY OBJECTIVES</font>

Insert objectives that must be completed to achieve the mission goal

<font color='#595305'>SECONDARY OBJECTIVES</font>

Insert objectives that are not required to complete the mission, but may be useful
"
                        .to_string(),
                    ),
                );
                Value::Object(map)
            },
            typ: MissionType::Contract,
        };
        let messages = briefing_messages(&mission);
        println!("{messages:#?}");
    }
}
