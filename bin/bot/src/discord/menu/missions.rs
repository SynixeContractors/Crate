use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandType, MessageId, ReactionType,
    },
    prelude::Context,
};
use synixe_events::missions::db::Response;
use synixe_meta::discord::role::STAFF;
use synixe_model::missions::aar::{Aar, PaymentType};
use synixe_proc::events_request;

use crate::discord::{
    interaction::{Confirmation, Generic, Interaction},
    utils::find_members,
};

pub fn aar_ids(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("AAR - Get IDs").kind(CommandType::Message)
}

pub async fn run_aar_ids(ctx: &Context, command: &ApplicationCommandInteraction) {
    let mut interaction = Interaction::new(ctx, Generic::Application(command));
    let Ok(msg) = command.channel_id
        .message(&ctx.http, MessageId::from(command.data.target_id.unwrap()))
        .await
    else {
        interaction
            .reply("Failed to find message")
            .await;
        return;
    };
    let Some(data) = msg.content.lines().into_iter().find(|l| l.starts_with("Contractors: ")) else {
        interaction
            .reply("Failed to find contractors list")
            .await;
        return;
    };
    let names = data
        .trim_start_matches("Contractors: ")
        .split(", ")
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();
    match find_members(ctx, &names).await {
        Ok(ids) => {
            let ids = ids.into_iter().map(|id| id.to_string()).collect::<Vec<_>>();
            interaction
                .reply(format!("**IDs**\n{}", ids.join("\n")))
                .await;
        }
        Err(e) => {
            interaction.reply(e).await;
        }
    }
}

pub fn aar_pay(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("AAR - Pay").kind(CommandType::Message)
}

#[allow(clippy::too_many_lines)]
pub async fn run_aar_pay(ctx: &Context, command: &ApplicationCommandInteraction) {
    let mut interaction = Interaction::new(ctx, Generic::Application(command));
    let Some(member) = command.member.as_ref() else {
        interaction
            .reply("Failed to get member")
            .await;
        return;
    };
    if !member.roles.contains(&STAFF) {
        interaction
            .reply("You must be staff to use this command")
            .await;
        return;
    }
    let Ok(message) = command.channel_id
        .message(&ctx.http, MessageId::from(command.data.target_id.unwrap()))
        .await
    else {
        interaction
            .reply("Failed to find message")
            .await;
        return;
    };
    let aar = match Aar::from_message(&message.content) {
        Ok(aar) => aar,
        Err(e) => {
            interaction.reply(format!(":confused: I couldn't parse that AAR. Please make sure you're using the template. Error: {e}")).await;
            return;
        }
    };
    match find_members(ctx, aar.contractors()).await {
        Ok(ids) => {
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
                    interaction
                        .reply("Failed to find scheduled date")
                        .await;
                    return;
                };
                let Some(payment) = interaction.choice("Select Payment Type", &PaymentType::as_choices()).await else {
                    interaction
                        .reply("Failed to get payment type")
                        .await;
                    return;
                };
                let Some(payment) = PaymentType::from_i32(payment.parse().unwrap()) else {
                    interaction
                        .reply("Failed to get payment type")
                        .await;
                    return;
                };
                if interaction
                    .confirm(&format!("```{}```", &aar.show_math(payment)))
                    .await
                    == Confirmation::Yes
                {
                    if let Ok(Ok((Response::PayMission(Ok(_)), _))) = events_request!(
                        bootstrap::NC::get().await,
                        synixe_events::missions::db,
                        PayMission {
                            scheduled: scheduled.id,
                            contractors: ids,
                            contractor_amount: aar.contractor_payment(payment),
                            group_amount: aar.employer_payment(payment),
                        }
                    )
                    .await
                    {
                        if let Err(e) = message
                            .reply(
                                &ctx.http,
                                format!(
                                    ":white_check_mark: **Paid**\n```{}```",
                                    aar.show_math(payment)
                                ),
                            )
                            .await
                        {
                            error!("Error replying to message: {}", e);
                        }
                        interaction.reply("Mission Paid").await;
                        if let Err(e) = message.react(&ctx.http, ReactionType::Unicode("✅".to_string())).await {
                            error!("Error reacting to message: {}", e);
                        }
                    } else {
                        interaction.reply("Failed to pay mission").await;
                    }
                } else {
                    interaction.reply("Mission not paid").await;
                }
            } else {
                interaction.reply("Failed to find scheduled date").await;
            }
        }
        Err(e) => {
            interaction.reply(e).await;
        }
    }
}
