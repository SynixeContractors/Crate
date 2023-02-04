use std::time::Duration;

use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        prelude::{
            application_command::CommandDataOption, autocomplete::AutocompleteInteraction,
            command::CommandOptionType, component::ButtonStyle,
            message_component::MessageComponentInteraction, InteractionResponseType, ReactionType,
        },
    },
    prelude::*,
};
use synixe_events::missions::db::Response;
use synixe_meta::discord::{
    channel::SCHEDULE,
    role::{MISSION_REVIEWER, STAFF},
};
use synixe_model::missions::{MissionRsvp, Rsvp, ScheduledMission};
use synixe_proc::events_request;
use time::format_description;
use time_tz::{timezones::db::america::NEW_YORK, OffsetDateTimeExt};

use crate::{
    discord::interaction::{Confirmation, Generic, Interaction},
    get_option,
};

use super::ShouldAsk;

const TIME_FORMAT: &str =
    "[year]-[month]-[day] [hour]:[minute] [offset_hour sign:mandatory]:[offset_minute]";

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
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
                .create_sub_option(|option| {
                    option
                        .name("channel")
                        .description("Channel to post the upcoming mission")
                        .kind(CommandOptionType::Channel)
                        .required(false)
                })
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for bank provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "new" => new(ctx, command, &subcommand.options).await?,
            "upcoming" => upcoming(ctx, command, &subcommand.options).await?,
            "remove" => remove(ctx, command, &subcommand.options).await?,
            "post" => post(ctx, command, &subcommand.options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub async fn autocomplete(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
) -> serenity::Result<()> {
    let Some(subcommand) = autocomplete.data.options.first() else {
        warn!("No subcommand for bank provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand && subcommand.name.as_str() == "new" {
        new_autocomplete(ctx, autocomplete, &subcommand.options).await?;
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub async fn rsvp_button(
    ctx: &Context,
    component: &MessageComponentInteraction,
) -> serenity::Result<()> {
    let channel = component.channel_id;
    let message = component.message.id;
    let Ok(Ok((Response::FetchScheduledMessage(Ok(Some(scheduled))), _))) =
        events_request!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            FetchScheduledMessage { channel, message }
        )
        .await else {
            error!("Failed to fetch scheduled mission for component");
            return Ok(());
        };
    match component.data.custom_id.as_str() {
        "rsvp_yes" => {
            let Ok(Ok((Response::AddMissionRsvp(Ok(())), _))) =
                events_request!(
                    bootstrap::NC::get().await,
                    synixe_events::missions::db,
                    AddMissionRsvp {
                        scheduled: scheduled.id,
                        member: component.user.id.to_string(),
                        rsvp: Rsvp::Yes,
                        details: None,
                    }
                )
                .await else {
                    error!("Failed to add mission rsvp for component");
                    return Ok(());
                };
            if let Err(e) = component
                .create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::DeferredUpdateMessage)
                })
                .await
            {
                error!("Failed to create interaction response: {}", e);
            }
        }
        "rsvp_maybe" => {
            let mut interaction = Interaction::new(ctx, Generic::Message(component), &[]);
            let Some(reason) = interaction
                .choice("Please provide a reason, this helps us make informed decision to improve Synixe!",
                &vec![
                    ("I might not be able to make it".to_string(), "not_sure".to_string()),
                    ("I'm not interested in this mission".to_string(), "not_interested".to_string()),
                    ("I'm burnt out and may not want to attend".to_string(), "burnt_out".to_string()),
                    ("Other".to_string(), "other".to_string()),
                ]
                )
                .await?
            else {
                warn!("No reason provided for rsvp_maybe");
                return Ok(());
            };
            let Ok(Ok((Response::AddMissionRsvp(Ok(())), _))) =
                events_request!(
                    bootstrap::NC::get().await,
                    synixe_events::missions::db,
                    AddMissionRsvp {
                        scheduled: scheduled.id,
                        member: component.user.id.to_string(),
                        rsvp: Rsvp::Maybe,
                        details: Some(reason),
                    }
                )
                .await else {
                    error!("Failed to add mission rsvp for component");
                    return Ok(());
                };
            interaction.reply("Thank you for your RSVP!").await?;
        }
        "rsvp_no" => {
            let mut interaction = Interaction::new(ctx, Generic::Message(component), &[]);
            let Some(reason) = interaction
                .choice("Please provide a reason, this helps us make informed decision to improve Synixe!",
                    &vec![
                        ("I won't be able to make it".to_string(), "not_sure".to_string()),
                        ("I'm not interested in this mission".to_string(), "not_interested".to_string()),
                        ("I'm burnt out".to_string(), "burnt_out".to_string()),
                        ("Other".to_string(), "other".to_string()),
                    ]
                )
                .await?
            else {
                warn!("No reason provided for rsvp_no");
                return Ok(());
            };
            let Ok(Ok((Response::AddMissionRsvp(Ok(())), _))) =
                events_request!(
                    bootstrap::NC::get().await,
                    synixe_events::missions::db,
                    AddMissionRsvp {
                        scheduled: scheduled.id,
                        member: component.user.id.to_string(),
                        rsvp: Rsvp::No,
                        details: Some(reason),
                    }
                )
                .await else {
                    error!("Failed to add mission rsvp for component");
                    return Ok(());
                };
            interaction.reply("Thank you for your RSVP!").await?;
        }
        _ => {
            warn!("Unknown component id: {}", component.data.custom_id);
        }
    }
    let Ok(Ok((Response::FetchMissionRsvps(Ok(rsvps)), _))) =
            events_request!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                FetchMissionRsvps { scheduled: scheduled.id }
            )
            .await
        else {
            return Ok(());
        };
    if let Err(e) = component
        .channel_id
        .edit_message(&ctx.http, message, |s| {
            s.embed(|e| {
                make_post_embed(e, &scheduled, &rsvps);
                e
            });
            s
        })
        .await
    {
        error!("Failed to edit message: {}", e);
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn new(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    let date = super::get_datetime(options);
    super::requires_roles(
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
        events_request!(
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
    let Ok(Ok((Response::FetchMissionList(Ok(missions)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionList {
            search: Some(mission_id.to_string()),
        }
    )
    .await else {
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
    autocomplete: &AutocompleteInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let Some(focus) = options.iter().find(|o| o.focused) else {
        return Ok(());
    };
    if focus.name != "mission" {
        return Ok(());
    }
    let Ok(Ok((Response::FetchMissionList(Ok(mut missions)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchMissionList {
            search: Some(focus.value.as_ref().expect("value should always exist").as_str().expect("discord should enforce string type").to_string())
        }
    )
    .await else {
        error!("failed to fetch mission list");
        return Ok(());
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
    Ok(())
}

async fn upcoming(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    match events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await
    {
        Ok(Ok((Response::UpcomingSchedule(Ok(upcoming)), _))) => {
            let mut content = String::from("**Upcoming Missions**\n\n");
            for scheduled in upcoming {
                content.push_str(&format!(
                    "**{}**\n<t:{}:F> - <t:{}:R>\n*{}*\n\n",
                    scheduled.name,
                    scheduled.start.unix_timestamp(),
                    scheduled.start.unix_timestamp(),
                    scheduled.summary,
                ));
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

#[allow(clippy::too_many_lines)]
pub async fn remove(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    super::requires_roles(
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
    let time_format =
        format_description::parse(TIME_FORMAT).expect("Time format should have been valid");
    debug!("fetching missions");
    interaction.reply("Fetching missions...").await?;
    let Ok(Ok((Response::UpcomingSchedule(Ok(missions)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await
    else {
        return interaction.reply("Failed to fetch missions").await;
    };
    let Some(scheduled_id) = interaction.choice("Select Mission", &missions.iter().map(|m| (format!(
        "{} - {}",
        m.mission,
        &m
            .start
            .to_timezone(NEW_YORK)
            .format(&time_format)
            .expect("Should have been able to format time")
    ), m.id)).collect()).await? else {
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
            &scheduled
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
            &scheduled
                .start
                .to_timezone(NEW_YORK)
                .format(&time_format)
                .expect("Should have been able to format time")
        ))
        .await?;
    match confirm {
        Confirmation::Yes => {
            if let Ok(Ok((Response::Unschedule(Ok(())), _))) = events_request!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                Unschedule {
                    scheduled: scheduled_id
                }
            )
            .await
            {
                if let Some((channel, message)) = &scheduled.message() {
                    if let Err(e) = channel.delete_message(&ctx, message).await {
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
                            .expect("Should have been able to format time")
                    ))
                    .await?;
            } else {
                interaction
                    .reply(format!(
                        "Failed to remove `{} - {}`",
                        scheduled.mission,
                        &scheduled
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
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    super::requires_roles(
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
    let Ok(Ok((Response::UpcomingSchedule(Ok(missions)), _))) = events_request!(
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
                    &format_description::parse(TIME_FORMAT)
                        .expect("Time format should have been valid")
                )
                .expect("Should have been able to format time")
        ))
        .await?;
    match confirm {
        Confirmation::Yes => {
            let Ok(Ok((Response::FetchMissionRsvps(Ok(rsvps)), _))) =
                events_request!(
                    bootstrap::NC::get().await,
                    synixe_events::missions::db,
                    FetchMissionRsvps { scheduled: scheduled.id }
                )
                .await
            else {
                return interaction.reply("Failed to fetch rsvps").await;
            };
            let channel =
                get_option!(options, "channel", Channel).map_or_else(|| SCHEDULE, |c| c.id);
            let Ok(sched) = channel
                .send_message(&ctx, |s| {
                    s.embed(|f| {
                        make_post_embed(f, scheduled, &rsvps);
                        f
                    });
                    s.components(|c| {
                        c.create_action_row(|r| {
                            r.create_button(|b| {
                                b.style(ButtonStyle::Secondary)
                                    .custom_id("rsvp_yes")
                                    .emoji(ReactionType::Unicode("🟩".to_string()))
                            })
                            .create_button(|b| {
                                b.style(ButtonStyle::Secondary)
                                    .custom_id("rsvp_maybe")
                                    .emoji(ReactionType::Unicode("🟨".to_string()))
                            })
                            .create_button(|b| {
                                b.style(ButtonStyle::Secondary)
                                    .custom_id("rsvp_no")
                                    .emoji(ReactionType::Unicode("🟥".to_string()))
                            })
                        })
                    })
                })
                .await else {
                    return interaction.reply("Failed to post mission").await;
                };
            tokio::time::sleep(Duration::from_millis(500)).await;
            let Ok(sched_thread) = channel
                .create_public_thread(&ctx, sched.id, |t| t.name(&scheduled.name))
                .await else {
                    return interaction.reply("Failed to create thread").await;
                };
            tokio::time::sleep(Duration::from_millis(100)).await;
            if let Err(e) = sched_thread
                .send_message(&ctx, |pt| {
                    pt.content(
                        scheduled
                            .description
                            .replace("            <br/>", "\n")
                            .replace("<font color='#D81717'>", "")
                            .replace("<font color='#1D69F6'>", "")
                            .replace("<font color='#993399'>", "")
                            .replace("<font color='#663300'>", "")
                            .replace("<font color='#139120'>", "")
                            .replace("</font color>", "") // felix you scoundrel
                            .replace("</font>", ""),
                    )
                })
                .await
            {
                error!("Failed to post mission description: {}", e);
            }
            if let Ok(Ok((Response::SetScheduledMesssage(Ok(())), _))) = events_request!(
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
                    .reply(format!("Posted `{}`", scheduled.mission))
                    .await?;
            } else {
                interaction
                    .reply(format!("Failed to post `{}`", scheduled.mission))
                    .await?;
            }
        }
        Confirmation::No => {
            interaction.reply("Cancelled mission posting").await?;
        }
        Confirmation::Timeout => {}
    }
    Ok(())
}

fn make_post_embed(embed: &mut CreateEmbed, scheduled: &ScheduledMission, rsvps: &[MissionRsvp]) {
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

    embed.title(&scheduled.name);
    embed.description(&scheduled.summary);
    embed.color(0x00ff_d731);
    embed.field(
        "🕒 Time",
        format!(
            "<t:{}:F> - <t:{}:R>",
            scheduled.start.unix_timestamp(),
            scheduled.start.unix_timestamp()
        ),
        false,
    );
    embed.field(
        format!("🟩 Attending ({})", yes.len()),
        {
            let out = yes
                .iter()
                .map(|r| format!("> <@{}>", r.member))
                .collect::<Vec<_>>()
                .join("\n");
            if out.is_empty() {
                "-".to_string()
            } else {
                out
            }
        },
        true,
    );
    embed.field(
        format!("🟨 Maybe ({})", maybe.len()),
        {
            let out = maybe
                .iter()
                .map(|r| format!("> <@{}>", r.member))
                .collect::<Vec<_>>()
                .join("\n");
            if out.is_empty() {
                "-".to_string()
            } else {
                out
            }
        },
        true,
    );
    embed.field(
        format!("🟥 Declined ({})", no.len()),
        {
            let out = no
                .iter()
                .map(|r| format!("> <@{}>", r.member))
                .collect::<Vec<_>>()
                .join("\n");
            if out.is_empty() {
                "-".to_string()
            } else {
                out
            }
        },
        true,
    );
}
