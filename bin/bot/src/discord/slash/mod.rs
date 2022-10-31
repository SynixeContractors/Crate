use serenity::{
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        InteractionResponseType, RoleId,
    },
    prelude::Context,
};
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use time_tz::{timezones::db::america::NEW_YORK, OffsetDateTimeExt, PrimitiveDateTimeExt};

pub mod meme;
pub mod missions;

pub async fn requires_role(
    needle: RoleId,
    haystack: &[RoleId],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> bool {
    let found = haystack.iter().any(|role| *role == needle);
    if !found {
        command
            .create_interaction_response(&ctx, |r| {
                r.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| {
                        m.content("You do not have permission to use this command")
                            .ephemeral(true)
                    })
            })
            .await
            .unwrap();
    }
    found
}

pub fn get_datetime(options: &[CommandDataOption]) -> OffsetDateTime {
    let month = options
        .iter()
        .find(|option| option.name == "month")
        .map_or_else(
            || {
                OffsetDateTime::now_utc()
                    .to_timezone(NEW_YORK)
                    .date()
                    .month()
            },
            |option| {
                let month: u8 = option
                    .value
                    .as_ref()
                    .unwrap()
                    .as_u64()
                    .unwrap()
                    .try_into()
                    .unwrap();
                month.try_into().unwrap()
            },
        );
    let day = options
        .iter()
        .find(|option| option.name == "day")
        .map_or_else(
            || OffsetDateTime::now_utc().to_timezone(NEW_YORK).date().day(),
            |option| {
                option
                    .value
                    .as_ref()
                    .unwrap()
                    .as_u64()
                    .unwrap()
                    .try_into()
                    .unwrap()
            },
        );
    let hour = options
        .iter()
        .find(|option| option.name == "hour")
        .map_or_else(
            || 22,
            |option| {
                option
                    .value
                    .as_ref()
                    .unwrap()
                    .as_u64()
                    .unwrap()
                    .try_into()
                    .unwrap()
            },
        );
    {
        let year = OffsetDateTime::now_utc()
            .to_timezone(NEW_YORK)
            .date()
            .year();
        let possible = PrimitiveDateTime::new(
            Date::from_calendar_date(year, month, day).unwrap(),
            Time::from_hms(hour, 0, 0).unwrap(),
        )
        .assume_timezone(NEW_YORK)
        .unwrap();
        if OffsetDateTime::now_utc().to_timezone(NEW_YORK) > possible {
            possible.replace_year(year + 1).unwrap()
        } else {
            possible
        }
    }
}
