use chrono::{Datelike, NaiveDateTime, TimeZone};
use chrono_tz::America::New_York;
use serenity::{
    model::prelude::{
        application_command::ApplicationCommandInteraction, InteractionResponseType, RoleId,
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

pub fn get_datetime(command: &ApplicationCommandInteraction) -> NaiveDateTime {
    let day = command
        .data
        .options
        .iter()
        .find(|option| option.name == "day")
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let hour = command
        .data
        .options
        .iter()
        .find(|option| option.name == "time")
        .map_or_else(
            || "22".to_string(),
            |option| option.value.as_ref().unwrap().as_str().unwrap().to_string(),
        );
    let (month, day) = day.split_once('-').unwrap();
    debug!("month: {}, day: {}", month, day);
    let date = {
        let year = chrono::Utc::now().with_timezone(&New_York).date().year();
        let possible = New_York
            .ymd(year, month.parse().unwrap(), day.parse().unwrap())
            .and_hms(hour.parse().unwrap(), 0, 0);
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
