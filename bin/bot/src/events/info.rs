use nats::asynk::Message;
use synixe_events::{discord::info, respond};

use crate::ArcCacheAndHttp;

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::too_many_lines)]
pub async fn handle(msg: Message, client: ArcCacheAndHttp) {
    let Ok((ev, _)) = synixe_events::parse_data!(msg, info::Request) else {
        return;
    };
    match ev {
        info::Request::Username { user } => {
            match synixe_meta::discord::GUILD
                .member(client.as_ref(), user)
                .await
            {
                Ok(member) => {
                    if let Err(e) = respond!(
                        msg,
                        info::Response::Username(Ok(info::Username {
                            response: if let Some(nick) = member.nick.as_ref() {
                                nick.clone()
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
        info::Request::MemberRoles { user } => {
            match synixe_meta::discord::GUILD
                .member(client.as_ref(), user)
                .await
            {
                Ok(member) => {
                    if let Err(e) =
                        respond!(msg, info::Response::MemberRoles(Ok(member.roles))).await
                    {
                        error!("Failed to respond to NATS: {}", e);
                    }
                }
                Err(e) => {
                    if let Err(e) =
                        respond!(msg, info::Response::MemberRoles(Err(e.to_string()))).await
                    {
                        error!("Failed to respond to NATS: {}", e);
                    }
                }
            }
        }
        info::Request::MemberByName { name } => {
            match synixe_meta::discord::GUILD
                .members(client.as_ref(), None, None)
                .await
            {
                Ok(members) => {
                    if let Err(e) = respond!(
                        msg,
                        info::Response::MemberByName(Ok(members
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
                        respond!(msg, info::Response::MemberByName(Err(e.to_string()))).await
                    {
                        error!("Failed to respond to NATS: {}", e);
                    }
                }
            }
        }
        info::Request::MembersByRole { role } => {
            match synixe_meta::discord::GUILD
                .members(client.as_ref(), None, None)
                .await
            {
                Ok(members) => {
                    if let Err(e) = respond!(
                        msg,
                        info::Response::MembersByRole(Ok(members
                            .iter()
                            .filter(|m| m.roles.contains(&role))
                            .cloned()
                            .collect()))
                    )
                    .await
                    {
                        error!("Failed to respond to NATS: {}", e);
                    }
                }
                Err(e) => {
                    if let Err(e) =
                        respond!(msg, info::Response::MembersByRole(Err(e.to_string()))).await
                    {
                        error!("Failed to respond to NATS: {}", e);
                    }
                }
            }
        }
    }
}
