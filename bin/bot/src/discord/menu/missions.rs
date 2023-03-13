use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandType, MessageId,
        ReactionType,
    },
    prelude::Context,
};
use synixe_events::{missions::db::Response, reputation};
use synixe_meta::discord::role::STAFF;
use synixe_model::missions::{
    aar::{Aar, PaymentType},
    MissionType,
};
use synixe_proc::events_request_2;
use time::macros::offset;

pub const MENU_AAR_PAY: &str = "AAR - Pay";
pub const MENU_AAR_IDS: &str = "AAR - Get IDs";

use crate::discord::{
    interaction::{Confirmation, Generic, Interaction},
    utils::find_members,
};

pub fn aar_ids(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name(MENU_AAR_IDS).kind(CommandType::Message)
}

pub async fn run_aar_ids(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), &[]);
    let Ok(msg) = command.channel_id
        .message(&ctx.http, MessageId::from(command.data.target_id.expect("Should only be possible to run this command on a message")))
        .await
    else {
        return interaction
            .reply("Failed to find message")
            .await;
    };
    let Some(data) = msg.content.lines().find(|l| l.starts_with("Contractors: ")) else {
        return interaction
            .reply("Failed to find contractors list")
            .await;
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
                .await
        }
        Err(e) => interaction.reply(e).await,
    }
}

pub fn aar_pay(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name(MENU_AAR_PAY).kind(CommandType::Message)
}

#[allow(clippy::too_many_lines)]
pub async fn run_aar_pay(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), &[]);
    let Some(member) = command.member.as_ref() else {
        return interaction
            .reply("Failed to get member")
            .await;
    };
    if !member.roles.contains(&STAFF) {
        return interaction
            .reply("You must be staff to use this command")
            .await;
    }
    let Ok(message) = command.channel_id
        .message(&ctx.http, MessageId::from(command.data.target_id.expect("Should only be possible to run this command on a message")))
        .await
    else {
        return interaction
            .reply("Failed to find message")
            .await;
    };
    let aar = match Aar::from_message(&message.content) {
        Ok(aar) => aar,
        Err(e) => {
            return interaction.reply(format!(":confused: I couldn't parse that AAR. Please make sure you're using the template. Error: {e}")).await;
        }
    };
    let Ok(Ok((reputation::db::Response::CurrentReputation(Ok(Some(Some(current_rep)))), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::reputation::db,
        CurrentReputation {
            at: aar.date().with_time(time::Time::from_hms(0, 0, 0).expect("Will always be valid date")).assume_offset(offset!(UTC)),
        }
    )
    .await else {
        return interaction.reply("Failed to get get reputation").await;
    };
    #[allow(clippy::cast_possible_truncation)]
    let current_rep = current_rep as f32;
    let Some(reputation) = interaction.choice(&format!("The current reputation is {current_rep:.0}. Select the reputation you want to use for this mission."), &vec![
        ("Extremely Postive", 150),
        ("Very Positive", 80),
        ("Positive", 30),
        ("Slightly Positive", 10),
        ("Neutral", 0),
        ("Slightly Negative", -10),
        ("Negative", -50),
        ("Very Negative", -100),
        ("Extremely Negative", -200),
    ].into_iter().map(
        |(name, rep)| (format!("{name} ({rep})"), rep)
    ).collect::<Vec<_>>()).await? else {
        return interaction
            .reply("Failed to get reputation")
            .await;
    };
    match find_members(ctx, aar.contractors()).await {
        Ok(ids) => {
            let Ok(Ok((Response::FindScheduledDate(Ok(Some(scheduled))), _))) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                FindScheduledDate {
                    mission: aar.mission().to_string(),
                    date: aar.date(),
                    subcon: aar.typ() == MissionType::SubContract,
                }
            )
            .await else {
                return interaction.reply("Failed to find scheduled date").await;
            };
            let Ok(Ok((reputation::db::Response::MissionCompleted(Ok(_)), _))) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::reputation::db,
                MissionCompleted { member: member.user.id, mission: scheduled.id.to_string(), reputation: reputation.parse().expect("Should only receive the number values from the reputation choices") }
            ).await else {
                return interaction.reply("Failed to set reputation").await;
            };
            let Some(payment) = interaction.choice("Select Payment Type", &PaymentType::as_choices()).await? else {
                return interaction
                    .reply("Failed to get payment type")
                    .await;
            };
            let Some(payment) = PaymentType::from_i32(payment.parse().expect("Should only receive the number values from the payment type choices")) else {
                return interaction
                    .reply("Failed to get payment type")
                    .await;
            };
            if interaction
                .confirm(&format!("```{}```", &aar.show_math(payment, current_rep)))
                .await?
                == Confirmation::Yes
            {
                if let Ok(Ok((Response::PayMission(Ok(_)), _))) = events_request_2!(
                    bootstrap::NC::get().await,
                    synixe_events::missions::db,
                    PayMission {
                        scheduled: scheduled.id,
                        contractors: ids,
                        contractor_amount: aar.contractor_payment(payment, current_rep),
                        group_amount: aar.employer_payment(payment, current_rep),
                    }
                )
                .await
                {
                    if let Err(e) = message
                        .reply(
                            &ctx.http,
                            format!(
                                ":white_check_mark: **Paid**\n```{}```",
                                aar.show_math(payment, current_rep)
                            ),
                        )
                        .await
                    {
                        error!("Error replying to message: {}", e);
                    }
                    if let Some((channel, message)) = scheduled.message() {
                        if let Some(thread) = channel.message(&ctx.http, message).await?.thread {
                            thread
                                .send_message(&ctx.http, |m| {
                                    m.content(format!(
                                        "Contractors Paid: {}```{}\n```",
                                        aar.show_math(payment, current_rep),
                                        aar.contractors()
                                            .iter()
                                            .map(|c| format!("<@{c}>"))
                                            .collect::<Vec<String>>()
                                            .join(", ")
                                    ))
                                })
                                .await?;
                            thread
                                .edit_thread(&ctx.http, |t| t.locked(true).archived(true))
                                .await?;
                        }
                    }
                    interaction.reply("Mission Paid").await?;
                    if let Err(e) = message
                        .react(&ctx.http, ReactionType::Unicode("✅".to_string()))
                        .await
                    {
                        error!("Error reacting to message: {}", e);
                    }
                } else {
                    interaction.reply("Failed to pay mission").await?;
                }
            } else {
                interaction.reply("Mission not paid").await?;
            }
        }
        Err(e) => {
            interaction.reply(e).await?;
        }
    }
    Ok(())
}
