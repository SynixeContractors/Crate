use serenity::{model::prelude::Message, prelude::Context};
use synixe_events::missions::db::Response;
use synixe_model::missions::{aar::Aar, MissionType};
use synixe_proc::events_request_2;

use crate::discord::utils::find_members;

pub async fn validate_aar(ctx: &Context, message: Message) {
    if !(message.content.starts_with("```") || message.content.ends_with("```")) {
        return;
    }
    if message.author.bot {
        return;
    }
    let aar = Aar::from_message(&message.content);
    let Ok(aar) = aar else {
        if let Err(e) = message.reply(&ctx.http, format!(":confused: I couldn't parse that AAR. Please make sure you're using the template. \n > {}", aar.expect_err("checked for ok"))).await {
            error!("Error replying to message: {}", e);
        };
        return;
    };
    if let Ok(Ok((Response::FindScheduledDate(Ok(scheduled)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FindScheduledDate {
            mission: aar.mission().to_string(),
            date: aar.date(),
            subcon: aar.typ() == MissionType::SubContract,
        }
    )
    .await
    {
        let Some(scheduled) = scheduled else {
            if let Err(e) = message.reply(&ctx.http, ":confused: I couldn't find that mission on that date. Double check the date and mission name.").await {
                error!("Error replying to message: {}", e);
            };
            return;
        };
        if let Err(e) = find_members(ctx, aar.contractors()).await {
            if let Err(e) = message.reply(&ctx.http, e).await {
                error!("Error replying to message: {}", e);
            };
            return;
        }
        if let Err(e) = message
            .reply(&ctx.http, ":white_check_mark: AAR Valid!")
            .await
        {
            error!("Error replying to message: {}", e);
        };
        if let Ok(Ok((Response::SetScheduledAar(Ok(())), _))) = events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            SetScheduledAar {
                scheduled: scheduled.id,
                message: message.id,
            }
        )
        .await
        {
            info!("Set AAR for scheduled {}", scheduled.id);
        } else {
            error!("Failed to set AAR for scheduled {}", scheduled.id);
        };
    } else if aar.typ() == MissionType::Contract {
        if let Err(e) = message.reply(&ctx.http, ":confused: I couldn't find that mission on that date. Double check the date and mission name.").await {
            error!("Error replying to message: {}", e);
        }
    } else if let Err(e) = message.reply(&ctx.http, ":white_check_mark: AAR Valid! Although since this is a non-contract mission, it won't be automatically linked to a mission.").await {
        error!("Error replying to message: {}", e);
    }
    let Ok((ids, unknown)) = find_members(ctx, aar.contractors()).await else {
        if let Err(e) = message.reply(&ctx.http, "Failed to find members").await {
            error!("Error replying to message: {}", e);
        };
        return;
    };
    if !unknown.is_empty() {
        if let Err(e) = message
            .reply(
                &ctx.http,
                format!(
                    "Could not find the following members: {}",
                    unknown.join(", ")
                ),
            )
            .await
        {
            error!("Error replying to message: {}", e);
            return;
        };
    }
    if let Err(e) = message
        .reply(&ctx.http, format!("Found {} members", ids.len()))
        .await
    {
        error!("Error replying to message: {}", e);
    }
}
