#![deny(clippy::unwrap_used)]

use nats::asynk::Message;
use opentelemetry::trace::Tracer;
use synixe_events::{discord::info, respond, Evokable};

use crate::Bot;

pub async fn handle(msg: Message, client: Bot) {
    let ((ev, _), pcx) = synixe_events::parse_data!(msg, info::Request);
    let _span = opentelemetry::global::tracer("bot").start_with_context(ev.name(), &pcx);
    match ev {
        info::Request::Username { user } => {
            match synixe_meta::discord::GUILD.member(&client, user).await {
                Ok(member) => {
                    if let Err(e) = respond!(
                        msg,
                        info::Response::Username(Ok(info::Username {
                            response: if let Some(nick) = member.nick.as_ref() {
                                nick.to_string()
                            } else {
                                member.user.name.clone()
                            },
                            nickname: member.nick,
                            user_name: member.user.name,
                        }))
                    )
                    .await
                    {
                        error!("Failed to respond to NATS: {}", e);
                    }
                }
                Err(e) => {
                    if let Err(e) =
                        respond!(msg, info::Response::Username(Err(e.to_string()))).await
                    {
                        error!("Failed to respond to NATS: {}", e);
                    }
                }
            }
        }
        info::Request::Roles { user } => {
            match synixe_meta::discord::GUILD.member(&client, user).await {
                Ok(member) => {
                    if let Err(e) = respond!(msg, info::Response::Roles(Ok(member.roles))).await {
                        error!("Failed to respond to NATS: {}", e);
                    }
                }
                Err(e) => {
                    if let Err(e) = respond!(msg, info::Response::Roles(Err(e.to_string()))).await {
                        error!("Failed to respond to NATS: {}", e);
                    }
                }
            }
        }
        info::Request::UserByName { name } => {
            match synixe_meta::discord::GUILD
                .members(&client.http, None, None)
                .await
            {
                Ok(members) => {
                    if let Err(e) = respond!(
                        msg,
                        info::Response::UserByName(Ok(members
                            .iter()
                            .find(|m| m.nick.as_ref() == Some(&name) || m.user.name == name)
                            .map(|m| m.user.id)))
                    )
                    .await
                    {
                        error!("Failed to respond to NATS: {}", e);
                    }
                }
                Err(e) => {
                    if let Err(e) =
                        respond!(msg, info::Response::UserByName(Err(e.to_string()))).await
                    {
                        error!("Failed to respond to NATS: {}", e);
                    }
                }
            }
        }
    }
}
