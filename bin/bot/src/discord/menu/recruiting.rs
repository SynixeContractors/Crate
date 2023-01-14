use serenity::{
    builder::CreateApplicationCommand,
    model::{
        application::interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        prelude::{command::CommandType, MessageId},
    },
    prelude::*,
};
use synixe_meta::discord::channel::RECRUITING;
use synixe_proc::events_request;

use crate::discord::interaction::{Generic, Interaction};

pub fn reply(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("Recruiting - Reply")
        .kind(CommandType::Message)
}

pub async fn run_reply(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), &[]);
    let Ok(msg) = RECRUITING
        .message(&ctx.http, MessageId::from(command.data.target_id.expect("Should only be possible to run this command on a message")))
        .await
    else {
        return interaction
            .reply("Failed to find message")
            .await;
    };
    if let Some(embed) = msg.embeds.first() {
        debug!("embeded url {:?}", embed.url);
        if let Some(url) = &embed.url {
            if url.starts_with("https://reddit.com") {
                let resp = if (events_request!(
                    bootstrap::NC::get().await,
                    synixe_events::recruiting::executions,
                    ReplyReddit {
                        url: url.to_string()
                    }
                )
                .await)
                    .is_ok()
                {
                    "Reply Sent"
                } else {
                    "Failed to send reply"
                };

                if let Err(why) = command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.content(resp))
                    })
                    .await
                {
                    error!("Cannot respond to slash command: {}", why);
                }
            } else if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .content("Only reddit posts can be replied to currently")
                                .ephemeral(true)
                        })
                })
                .await
            {
                error!("Cannot respond to slash command: {}", why);
            }
        }
    }
    Ok(())
}
