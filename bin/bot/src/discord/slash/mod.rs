use chrono::{Datelike, NaiveDateTime, TimeZone};
use chrono_tz::America::New_York;
use serenity::{
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        InteractionResponseType, RoleId,
    },
    prelude::Context,
};

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

pub fn get_datetime(options: &[CommandDataOption]) -> NaiveDateTime {
    let month = options
        .iter()
        .find(|option| option.name == "month")
        .map_or_else(
            || chrono::Utc::now().with_timezone(&New_York).date().month(),
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
    let day = options
        .iter()
        .find(|option| option.name == "day")
        .map_or_else(
            || chrono::Utc::now().with_timezone(&New_York).date().day(),
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
    let date = {
        let year = chrono::Utc::now().with_timezone(&New_York).date().year();
        let possible = New_York.ymd(year, month, day).and_hms(hour, 0, 0);
        if chrono::Utc::now()
            .signed_duration_since(possible)
            .num_seconds()
            > 0
        {
            possible.with_year(year + 1).unwrap()
        } else {
            possible
        }
    };
    date.naive_utc()
}
