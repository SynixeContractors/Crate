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

pub fn reply(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("Recruiting - Reply")
        .kind(CommandType::Message)
}

pub async fn run_reply(ctx: &Context, command: &ApplicationCommandInteraction) {
    if let Ok(msg) = RECRUITING
        .message(&ctx.http, MessageId::from(command.data.target_id.unwrap()))
        .await
    {
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
                                message.content("Only reddit posts can be replied to currently")
                            })
                    })
                    .await
                {
                    error!("Cannot respond to slash command: {}", why);
                }
            }
        }
    } else if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("This command can only be used on recruiting posts")
                })
        })
        .await
    {
        error!("Cannot respond to slash command: {}", why);
    }
}
