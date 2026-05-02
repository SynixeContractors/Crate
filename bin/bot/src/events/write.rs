use async_nats::message::Message;
use serenity::{
    all::{CreateThread, EditThread, MessageId},
    builder::CreateMessage,
    model::prelude::ChannelId,
};
use synixe_events::{
    discord::write::{self, DiscordMessage},
    respond,
};
use synixe_meta::discord::{
    GUILD,
    channel::{GAME_LOG, LOG, MISSION_MAKING},
};
use synixe_proc::events_request_5;

use crate::ArcCacheAndHttp;

#[allow(clippy::cognitive_complexity)]
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
                    } else if let Err(e) = respond!(msg, write::Response::UserMessage(Ok(()))).await
                    {
                        error!("Failed to respond to NATS: {}", e);
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
            let message = audit(client, message, LOG).await;
            if let Some(message) = message {
                if let Err(e) = respond!(msg, write::Response::Audit(Ok((LOG, message)))).await {
                    error!("Failed to respond to NATS: {}", e);
                }
            } else if let Err(e) = respond!(
                msg,
                write::Response::Audit(Err(String::from("Failed to send message")))
            )
            .await
            {
                error!("Failed to respond to NATS: {}", e);
            }
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
        write::Request::PullRequestThreadMessage {
            number,
            title,
            content,
        } => {
            let Some(channel) = ensure_pr_thread(client.clone(), number, title).await else {
                error!("Failed to ensure PR thread");
                return;
            };
            if let Err(e) = channel
                .send_message(
                    client.as_ref(),
                    match content {
                        write::DiscordContent::Text(text) => CreateMessage::default().content(text),
                        write::DiscordContent::Embed(embed) => {
                            CreateMessage::default().embed(embed.into())
                        }
                    },
                )
                .await
            {
                error!("Failed to send message: {}", e);
            }
            respond!(msg, write::Response::PullRequestThreadMessage(Ok(())))
                .await
                .unwrap_or_else(|e| error!("Failed to respond to NATS: {}", e));
        }
        write::Request::PullRequestThreadUser {
            number,
            title,
            user,
            reason,
        } => {
            let Some(channel) = ensure_pr_thread(client.clone(), number, title).await else {
                error!("Failed to ensure PR thread");
                return;
            };
            if let Err(e) = channel.add_thread_member(client.as_ref(), user).await {
                error!("Failed to add member to thread: {}", e);
            }
            if let Err(e) = channel
                .send_message(
                    client.as_ref(),
                    CreateMessage::default()
                        .content(format!("Added <@{user}> to the thread: {reason}")),
                )
                .await
            {
                error!("Failed to send message: {}", e);
            }
            respond!(msg, write::Response::PullRequestThreadUser(Ok(())))
                .await
                .unwrap_or_else(|e| error!("Failed to respond to NATS: {}", e));
        }
    }
}

async fn ensure_pr_thread(client: ArcCacheAndHttp, number: i32, name: String) -> Option<ChannelId> {
    let Ok(Ok((synixe_events::github::db::Response::GetPullRequestThread(Ok(thread)), _))) =
        events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::github::db,
            GetPullRequestThread { number }
        )
        .await
    else {
        error!("Failed to get pull request thread from database");
        return None;
    };

    #[allow(clippy::cast_sign_loss)]
    if let Some(thread) = thread {
        let thread_id = ChannelId::new(thread.parse::<u64>().unwrap_or(0));

        let thread = thread_id
            .edit_thread(client.1, EditThread::default().name(name))
            .await;
        if let Err(e) = thread {
            error!("Failed to edit thread: {}", e);
        }

        return Some(thread_id);
    }

    // Create thread
    match MISSION_MAKING
        .create_thread(client.1, CreateThread::new(name))
        .await
    {
        Ok(channel) => {
            let channel = channel.id;
            // Save thread to database
            if let Err(e) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::github::db,
                SavePullRequestThread {
                    number,
                    thread_id: channel,
                }
            )
            .await
            {
                error!("Failed to save pull request thread to database: {}", e);
                return None;
            }
            Some(channel)
        }
        Err(e) => {
            error!("Failed to create thread: {}", e);
            None
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
