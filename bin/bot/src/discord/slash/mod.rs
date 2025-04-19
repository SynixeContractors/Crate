use std::time::Duration;

use serenity::{
    all::{
        ButtonStyle, CommandDataOption, CommandDataOptionValue, CommandOptionType, RoleId, UserId,
    },
    builder::{
        CreateActionRow, CreateButton, CreateCommandOption, CreateInteractionResponse,
        CreateMessage, EditMessage,
    },
};
use synixe_meta::discord::channel::LOG;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, Weekday};
use time_tz::{timezones::db::america::NEW_YORK, OffsetDateTimeExt, PrimitiveDateTimeExt};

use crate::{bot::Bot, cache_http::CacheAndHttp};

use super::interaction::{Confirmation, Interaction};

pub mod bank;
pub mod certifications;
pub mod docker;
pub mod garage;
pub mod gear;
pub mod missions;
pub mod reputation;
pub mod reset;
pub mod schedule;
pub mod surveys;

#[derive(Debug, Clone)]
pub enum ShouldAsk<'a> {
    /// Ask the #log channel for permission
    Yes((&'static str, &'a [CommandDataOption])),
    /// Do not ask the #log channel for permission, deny the command
    Deny,
}

pub async fn requires_roles(
    user: UserId,
    needle: &[RoleId],
    haystack: &[RoleId],
    ask: ShouldAsk<'_>,
    interaction: &mut Interaction<'_>,
) -> serenity::Result<()> {
    if !haystack.iter().any(|role| needle.contains(role)) {
        if let ShouldAsk::Yes((name, options)) = ask {
            let command = format!(
                "`/{name} {}`\n",
                options
                    .iter()
                    .map(|option| format!(
                        "{}: `{}`\n",
                        option.name.clone(),
                        match &option.value {
                            CommandDataOptionValue::Autocomplete { .. } =>
                                "Autocomplete".to_string(),
                            CommandDataOptionValue::Boolean(b) => b.to_string(),
                            CommandDataOptionValue::Integer(i) => i.to_string(),
                            CommandDataOptionValue::Number(n) => n.to_string(),
                            CommandDataOptionValue::String(s) => s.clone(),
                            CommandDataOptionValue::SubCommand(_) => "SubCommand".to_string(),
                            CommandDataOptionValue::SubCommandGroup(_) =>
                                "SubCommandGroup".to_string(),
                            CommandDataOptionValue::Attachment(a) => format!("Attachment: {a}"),
                            CommandDataOptionValue::Channel(c) => format!("<#{c}>"),
                            CommandDataOptionValue::Mentionable(m) => m.to_string(),
                            CommandDataOptionValue::Role(r) => format!("<@&{r}>"),
                            CommandDataOptionValue::User(u) => format!("<@{u}>"),
                            CommandDataOptionValue::Unknown(uk) => format!("Unknown: {uk}"),
                            _ => "Unrecognized".to_string(),
                        }
                    ))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            if interaction.confirm(
                "You do not have permission to use this command. Would you like to request permission?"
            ).await? == Confirmation::Yes {
                let Ok(mut message) = LOG.send_message(CacheAndHttp::get().as_ref(), {
                    CreateMessage::default().content(format!("<@{user}> is requesting permission to use {command}"))
                    .components(vec![
                        CreateActionRow::Buttons(vec![
                            CreateButton::new("approve")
                                .style(ButtonStyle::Danger)
                                .label("Approve"),
                            CreateButton::new("deny")
                                .style(ButtonStyle::Secondary)
                                .label("Deny")
                        ])
                    ])
                }).await else {
                    interaction.reply("Failed to send request message.").await?;
                    return Err(serenity::Error::Other("Failed to send request message"));
                };
                interaction.reply("Requesting permission... Staff have 10 minutes to approve your request.").await?;
                let Some(confirm_interaction) = message
                    .await_component_interaction(&*Bot::get())
                    .timeout(Duration::from_secs(60 * 10))
                    .next()
                    .await
                else {
                    message.reply_ping(CacheAndHttp::get().as_ref(), "Request timed out").await?;
                    interaction.reply("Didn't receive a response").await?;
                    return Err(serenity::Error::Other("Didn't receive a response"));
                };
                confirm_interaction
                    .create_response(CacheAndHttp::get().as_ref(), CreateInteractionResponse::Acknowledge)
                    .await?;
                message.edit(CacheAndHttp::get().as_ref(), EditMessage::default().components(vec![])).await?;
                if confirm_interaction.data.custom_id == "approve" {
                    message.reply(CacheAndHttp::get().as_ref(), format!("Approved by <@{}>", confirm_interaction.user.id)).await?;
                    return Ok(());
                }
                message.reply(CacheAndHttp::get().as_ref(), format!("Denied by <@{}>", confirm_interaction.user.id)).await?;
                interaction.reply("Denied").await?;
                return Err(serenity::Error::Other("Denied"));
            }
        } else {
            interaction
                .reply("You do not have permission to use this command.")
                .await?;
            return Err(serenity::Error::Other("Denied"));
        }
    }
    Ok(())
}

pub fn macro_get_option<'a>(
    options: &'a [CommandDataOption],
    name: &str,
) -> Option<&'a CommandDataOptionValue> {
    let option = options.iter().find(|option| option.name == name)?;
    Some(&option.value)
}

#[macro_export]
macro_rules! get_option {
    ($options:expr, $name:expr, $typ:ident) => {
        match $crate::discord::slash::macro_get_option($options, $name) {
            Some(serenity::all::CommandDataOptionValue::$typ(data)) => Some(data),
            _ => None,
        }
    };
}

#[macro_export]
macro_rules! get_option_user {
    ($options:expr, $name:expr) => {
        match $crate::discord::slash::macro_get_option($options, $name) {
            Some(serenity::all::CommandDataOptionValue::User(user)) => Some(user),
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
        || {
            if matches!(
                OffsetDateTime::now_utc()
                    .to_timezone(NEW_YORK)
                    .replace_day(day)
                    .expect("The day will be valid cause we check it above")
                    .replace_month(month)
                    .expect("The month will be valid cause we check it above")
                    .weekday(),
                Weekday::Saturday | Weekday::Sunday
            ) {
                16
            } else {
                22
            }
        },
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
    fn allow_public(self) -> Self;
}

impl AllowPublic for CreateCommandOption {
    fn allow_public(self) -> Self {
        self.add_sub_option(
            Self::new(
                CommandOptionType::Boolean,
                "public",
                "Post the response publicly",
            )
            .required(false),
        )
    }
}
