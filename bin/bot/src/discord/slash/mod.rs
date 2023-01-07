use serenity::model::prelude::{
    application_command::{CommandDataOption, CommandDataOptionValue},
    RoleId,
};
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use time_tz::{timezones::db::america::NEW_YORK, OffsetDateTimeExt, PrimitiveDateTimeExt};

use super::interaction::Interaction;

pub mod bank;
pub mod certifications;
pub mod dlc;
pub mod meme;
pub mod missions;

pub async fn requires_role(
    needle: RoleId,
    haystack: &[RoleId],
    interaction: &mut Interaction<'_>,
) -> serenity::Result<()> {
    if !haystack.iter().any(|role| *role == needle) {
        interaction
            .reply("You do not have permission to use this command.")
            .await?;
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
            Some(CommandDataOptionValue::User(user, _member)) => Some(user),
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
