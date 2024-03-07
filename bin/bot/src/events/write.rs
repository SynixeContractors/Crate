use nats::asynk::Message;
use serenity::{all::MessageId, builder::CreateMessage, model::prelude::ChannelId};
use synixe_events::{
    discord::write::{self, DiscordMessage},
    respond,
};
use synixe_meta::discord::{
    channel::{GAME_LOG, LOG},
    GUILD,
};

use crate::ArcCacheAndHttp;

#[allow(clippy::too_many_lines)]
pub async fn handle(msg: Message, client: ArcCacheAndHttp) {
    let Ok((ev, _)) = synixe_events::parse_data!(msg, write::Request) else {
        return;
    };
    match ev {
        write::Request::ChannelMessage {
            channel,
            message,
            thread: _,
        } => {
            if let Err(e) = channel
                .send_message(
                    client.as_ref(),
                    match message.content {
                        write::DiscordContent::Text(text) => CreateMessage::default().content(text),
                        write::DiscordContent::Embed(embed) => {
                            CreateMessage::default().embed(embed.into())
                        }
                    },
                )
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
            let dm = user.create_dm_channel(client.as_ref()).await;
            match dm {
                Ok(dm) => {
                    if let Err(e) = dm
                        .send_message(
                            client.as_ref(),
                            match message.content {
                                write::DiscordContent::Text(text) => {
                                    CreateMessage::default().content(text)
                                }
                                write::DiscordContent::Embed(embed) => {
                                    CreateMessage::default().embed(embed.into())
                                }
                            },
                        )
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
            let Ok(()) = respond!(msg, write::Response::EnsureRoles(Ok(()))).await else {
                error!("Failed to respond to NATS");
                return;
            };
            let Ok(member) = GUILD.member(client.as_ref(), member).await else {
                error!("Failed to get member");
                if let Err(e) = respond!(
                    msg,
                    write::Response::EnsureRoles(Err(String::from("Failed to get member")))
                )
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
                if let Err(e) = member.add_role(client.as_ref(), role).await {
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
        write::Request::Audit { message } => {
            let Ok(()) = respond!(msg, write::Response::Audit(Ok(()))).await else {
                error!("Failed to respond to NATS");
                return;
            };
            audit(client, message, LOG).await;
        }
        write::Request::GameAudit { message } => {
            let message = audit(client, message, GAME_LOG).await;
            if let Some(message) = message {
                if let Err(e) =
                    respond!(msg, write::Response::GameAudit(Ok((GAME_LOG, message)))).await
                {
                    error!("Failed to respond to NATS: {}", e);
                }
            } else if let Err(e) = respond!(
                msg,
                write::Response::GameAudit(Err(String::from("Failed to send message")))
            )
            .await
            {
                error!("Failed to respond to NATS: {}", e);
            }
        }
    }
}

async fn audit(
    client: ArcCacheAndHttp,
    message: DiscordMessage,
    channel: ChannelId,
) -> Option<MessageId> {
    if let Ok(m) = channel
        .send_message(
            client.as_ref(),
            match message.content {
                write::DiscordContent::Text(text) => CreateMessage::default().content(text),
                write::DiscordContent::Embed(embed) => CreateMessage::default().embed(embed.into()),
            },
        )
        .await
    {
        Some(m.id)
    } else {
        None
    }
}
