use std::time::Duration;

use serenity::{
    builder::CreateApplicationCommandOption,
    model::prelude::{
        application_command::{CommandDataOption, CommandDataOptionValue},
        command::CommandOptionType,
        component::ButtonStyle,
        InteractionResponseType, RoleId, UserId,
    },
};
use synixe_meta::discord::channel::LOG;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use time_tz::{timezones::db::america::NEW_YORK, OffsetDateTimeExt, PrimitiveDateTimeExt};

use crate::{bot::Bot, cache_http::CacheAndHttp};

use super::interaction::{Confirmation, Interaction};

pub mod bank;
pub mod certifications;
pub mod docker;
pub mod garage;
pub mod meme;
pub mod missions;
pub mod reputation;
pub mod schedule;

#[derive(Debug, Clone)]
pub enum ShouldAsk<'a> {
    /// Ask the #log channel for permission
    Yes((&'static str, &'a [CommandDataOption])),
    /// Do not ask the #log channel for permission, deny the command
    Deny,
}

pub async fn requires_roles<'a>(
    user: UserId,
    needle: &[RoleId],
    haystack: &[RoleId],
    ask: ShouldAsk<'a>,
    interaction: &mut Interaction<'_>,
) -> serenity::Result<()> {
    if !haystack.iter().any(|role| needle.contains(role)) {
        if let ShouldAsk::Yes((name, options)) = ask {
            let command = format!(
                "{name} {}",
                options
                    .iter()
                    .map(|option| format!(
                        "{}: `{}`",
                        option.name.clone(),
                        option.value.as_ref().expect("value")
                    ))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            if interaction.confirm(
                "You do not have permission to use this command. Would you like to request permission?"
            ).await? == Confirmation::Yes {
                let Ok(mut message) = LOG.send_message(&*CacheAndHttp::get(), |f| {
                    f.content(format!("<@{user}> is requesting permission to use {command}"))
                    .components(|c| {
                        c.create_action_row(|r| {
                            r.create_button(|b| {
                                b.style(ButtonStyle::Danger)
                                .label("Approve")
                                .custom_id("approve")
                            })
                            .create_button(|b| {
                                b.style(ButtonStyle::Secondary)
                                .label("Deny")
                                .custom_id("deny")
                            })
                        })
                    })
                }).await else {
                    interaction.reply("Failed to send request message.").await?;
                    return Err(serenity::Error::Other("Failed to send request message"));
                };
                interaction.reply("Requesting permission... Staff have 5 minutes to approve your request.").await?;
                let Some(confirm_interaction) = message
                    .await_component_interaction(&*Bot::get())
                    .timeout(Duration::from_secs(60 * 5))
                    .collect_limit(1)
                    .await
                else {
                    interaction.reply("Didn't receive a response").await?;
                    return Err(serenity::Error::Other("Didn't receive a response"));
                };
                confirm_interaction
                    .create_interaction_response(&*CacheAndHttp::get(), |r| {
                        r.kind(InteractionResponseType::DeferredUpdateMessage)
                    })
                    .await?;
                message.edit(&*CacheAndHttp::get(), |f| {
                    f.components(|c| {
                        c
                    })
                }).await?;
                if confirm_interaction.data.custom_id == "approve" {
                    message.reply(&*CacheAndHttp::get(), format!("Approved by <@{}>", confirm_interaction.user.id)).await?;
                    return Ok(());
                }
                message.reply(&*CacheAndHttp::get(), format!("Denied by <@{}>", confirm_interaction.user.id)).await?;
                interaction.reply("Denied").await?;
                return Err(serenity::Error::Other("Denied"));
            };
        } else {
            interaction
                .reply("You do not have permission to use this command.")
                .await?;
            return Err(serenity::Error::Other("Denied"));
        }
    }
    Ok(())
}

pub fn _get_option<'a>(
    options: &'a [CommandDataOption],
    name: &str,
) -> Option<&'a CommandDataOptionValue> {
    let option = options.iter().find(|option| option.name == name)?;
    option.resolved.as_ref()
}

#[macro_export]
macro_rules! get_option {
    ($options:expr, $name:expr, $typ:ident) => {
        match $crate::discord::slash::_get_option($options, $name) {
            Some(serenity::model::prelude::application_command::CommandDataOptionValue::$typ(
                data,
            )) => Some(data),
            _ => None,
        }
    };
}

#[macro_export]
macro_rules! get_option_user {
    ($options:expr, $name:expr) => {
        match $crate::discord::slash::_get_option($options, $name) {
            Some(serenity::model::prelude::application_command::CommandDataOptionValue::User(
                user,
                _member,
            )) => Some(user),
            _ => None,
        }
    };
}

pub fn get_datetime(options: &[CommandDataOption]) -> OffsetDateTime {
    let month = get_option!(options, "month", Integer).map_or_else(
        || {
            OffsetDateTime::now_utc()
                .to_timezone(NEW_YORK)
                .date()
                .month()
        },
        |month| {
            let month: u8 = (*month).try_into().expect("Discord should limit to 1-12");
            month
                .try_into()
                .expect("Month should never be invalid at this point")
        },
    );
    let day = get_option!(options, "day", Integer).map_or_else(
        || OffsetDateTime::now_utc().to_timezone(NEW_YORK).date().day(),
        |day| (*day).try_into().expect("Discord should limit to 1-31"),
    );
    let hour = get_option!(options, "hour", Integer).map_or_else(
        || 22,
        |hour| (*hour).try_into().expect("Discord should limit to 0-23"),
    );
    {
        let year = OffsetDateTime::now_utc()
            .to_timezone(NEW_YORK)
            .date()
            .year();
        let possible = PrimitiveDateTime::new(
            Date::from_calendar_date(year, month, day).expect("Date should be valid"),
            Time::from_hms(hour, 0, 0).expect("Time should be valid"),
        )
        .assume_timezone(NEW_YORK)
        .unwrap();
        if OffsetDateTime::now_utc().to_timezone(NEW_YORK) > possible {
            possible.replace_year(year + 1).expect("Year overflowed")
        } else {
            possible
        }
    }
}

pub trait AllowPublic {
    fn allow_public(&mut self) -> &mut Self;
}

impl AllowPublic for CreateApplicationCommandOption {
    fn allow_public(&mut self) -> &mut Self {
        self.create_sub_option(|option| {
            option
                .name("public")
                .description("Post the response publicly")
                .kind(CommandOptionType::Boolean)
                .required(false)
        })
    }
}
