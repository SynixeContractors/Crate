use serenity::{model::prelude::Message, prelude::Context};
use synixe_events::missions::db::Response;
use synixe_model::missions::aar::Aar;
use synixe_proc::events_request;

use crate::discord::utils::find_members;

pub async fn validate_aar(ctx: &Context, message: Message) {
    if !(message.content.starts_with("```") || message.content.ends_with("```")) {
        return;
    }
    if message.author.bot {
        return;
    }
    let Ok(aar) = Aar::from_message(&message.content) else {
        if let Err(e) = message.reply(&ctx.http, ":confused: I couldn't parse that AAR. Please make sure you're using the template.").await {
            error!("Error replying to message: {}", e);
        };
        return;
    };
    if let Ok(Ok((Response::FindScheduledDate(Ok(scheduled)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FindScheduledDate {
            mission: aar.mission().to_string(),
            date: aar.date(),
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
        if let Ok(Ok((Response::SetScheduledAar(Ok(())), _))) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            SetScheduledAar {
                scheduled: scheduled.id,
                message_id: message.id,
            }
        )
        .await
        {
            info!("Set AAR for scheduled {}", scheduled.id);
        } else {
            error!("Failed to set AAR for scheduled {}", scheduled.id);
        };
    }
}
