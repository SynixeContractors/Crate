#![deny(clippy::unwrap_used)]

use nats::asynk::Message;
use opentelemetry::trace::Tracer;
use synixe_events::{discord::write, respond, Evokable};
use synixe_meta::discord::GUILD;

use crate::Bot;

pub async fn handle(msg: Message, client: Bot) {
    let ((ev, _), pcx) = synixe_events::parse_data!(msg, write::Request);
    let _span = opentelemetry::global::tracer("bot").start_with_context(ev.name(), &pcx);
    match ev {
        write::Request::ChannelMessage {
            channel,
            message,
            thread: _,
        } => {
            if let Err(e) = channel
                .send_message(&client.http, |m| match message.content {
                    write::DiscordContent::Text(text) => m.content(text),
                    write::DiscordContent::Embed(embed) => m.set_embed(embed.into()),
                })
                .await
            {
                error!("Failed to send message: {}", e);
                if let Err(e) =
                    respond!(msg, write::Response::ChannelMessage(Err(e.to_string()))).await
                {
                    error!("Failed to respond to NATS: {}", e);
                }
            } else if let Err(e) = respond!(msg, &write::Response::ChannelMessage(Ok(()))).await {
                error!("Failed to respond to NATS: {}", e);
            }
        }
        write::Request::UserMessage { user, message } => {
            let dm = user.create_dm_channel(&client.http).await;
            match dm {
                Ok(dm) => {
                    if let Err(e) = dm
                        .send_message(&client.http, |m| match message.content {
                            write::DiscordContent::Text(text) => m.content(text),
                            write::DiscordContent::Embed(embed) => m.set_embed(embed.into()),
                        })
                        .await
                    {
                        if let Err(e) =
                            respond!(msg, write::Response::UserMessage(Err(e.to_string()))).await
                        {
                            error!("Failed to respond to NATS: {}", e);
                        }
                    }
                }
                Err(e) => {
                    if let Err(e) =
                        respond!(msg, write::Response::UserMessage(Err(e.to_string()))).await
                    {
                        error!("Failed to respond to NATS: {}", e);
                    }
                }
            }
        }
        write::Request::EnsureRoles { member, roles } => {
            respond!(msg, write::Response::EnsureRoles(Ok(()))).await.unwrap();
            let Ok(mut member) = GUILD.member(&client, member).await else {
                error!("Failed to get member");
                if let Err(e) = respond!(msg, write::Response::EnsureRoles(Err(String::from(
                    "Failed to get member"
                ))))
                .await
                {
                    error!("Failed to respond to NATS: {}", e);
                }
                return;
            };
            let roles = roles
                .iter()
                .filter(|r| !member.roles.contains(r))
                .collect::<Vec<_>>();
            for role in roles {
                if let Err(e) = member.add_role(&client.http, role).await {
                    error!("Failed to add role: {}", e);
                    if let Err(e) =
                        respond!(msg, write::Response::EnsureRoles(Err(e.to_string()))).await
                    {
                        error!("Failed to respond to NATS: {}", e);
                    }
                    return;
                }
            }
        }
    }
}
