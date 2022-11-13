use arma_rs::Group;
use serenity::model::prelude::UserId;
use synixe_events::discord::{self, db, info};
use synixe_proc::events_request;

use crate::{models::discord::MemberInfo, CONTEXT, RUNTIME, STEAM_CACHE};

pub fn group() -> Group {
    Group::new()
        .command("member", command_member)
        .command("guild", command_guild)
}

/// Fetches a user's discord id and roles
fn command_member(steam: String, name: String) {
    RUNTIME.spawn(async move {
        let Ok(((db::Response::FromSteam(resp), _), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::discord::db,
            FromSteam {
                steam_id: steam.clone(),
            }
        ).await else {
            error!("failed to fetch discord id over nats");
            return;
        };
        let discord_id = if let Ok(discord_id) = resp { discord_id } else {
            let Ok(((discord::info::Response::MemberByName(Ok(Some(discord_id))), _), _)) = events_request!(
                bootstrap::NC::get().await,
                synixe_events::discord::info,
                MemberByName {
                    name: name.clone(),
                }
            ).await else {
                error!("failed to check for name match over nats");
                CONTEXT.read().await.as_ref().unwrap().callback_data("crate_server", "needs_link", vec![steam.clone()]);
                return;
            };
            let Ok(((db::Response::SaveSteam(Ok(())), _), _)) = events_request!(
                bootstrap::NC::get().await,
                synixe_events::discord::db,
                SaveSteam {
                    steam_id: steam.clone(),
                    member: discord_id,
                }
            ).await else {
                error!("failed to save discord id over nats");
                return;
            };
            discord_id.to_string()
        };
        let Ok(((info::Response::MemberRoles(resp), _), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::discord::info,
            MemberRoles {
                user: UserId(discord_id.parse().unwrap()),
            }
        ).await else {
            error!("failed to fetch discord roles over nats");
            return;
        };
        let Ok(roles) = resp else {
            error!("failed to fetch discord roles over nats");
            return;
        };
        STEAM_CACHE.write().await.insert(discord_id.clone(), steam.clone());
        CONTEXT.read().await.as_ref().unwrap().callback_data("crate_server:discord", "member", MemberInfo {
            steam,
            discord_id,
            roles: roles.into_iter().map(|r| r.to_string()).collect(),
        });
    });
}

fn command_guild() {
    RUNTIME.spawn(async move {});
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use arma_rs::Value;

    #[test]
    fn fetch_brett() {
        std::env::set_var("CRATE_SERVER_ID", "test_fetch_brett");
        let ext = super::super::init().testing();
        unsafe {
            let (_, code) = ext.call(
                "discord:fetch",
                Some(vec![String::from("76561198076832016")]),
            );
            assert_eq!(code, 0);
        }
        assert!(ext
            .callback_handler(
                |name, func, data| {
                    if name == "crate_log" {
                        println!("{}: {}", func, data.unwrap());
                        return arma_rs::Result::<(), ()>::Continue;
                    }
                    assert_eq!(name, "crate_server");
                    assert_eq!(func, "fetch");
                    let Value::Array(data) = data.unwrap() else {
                        panic!("expected array");
                    };
                    assert_eq!(data[0], Value::String(String::from("307524009854107648")));
                    let Value::Array(roles) = data[1].clone() else {
                        panic!("expected array");
                    };
                    assert!(roles.contains(&Value::String(String::from("700888852142751815"))));
                    arma_rs::Result::Ok(())
                },
                Duration::from_secs(10)
            )
            .is_ok());
    }

    #[test]
    fn fetch_missing() {
        std::env::set_var("CRATE_SERVER_ID", "test_fetch_brett");
        let ext = super::super::init().testing();
        unsafe {
            let (_, code) = ext.call("discord:fetch", Some(vec![String::from("0123")]));
            assert_eq!(code, 0);
        }
        assert!(ext
            .callback_handler(
                |name, func, data| {
                    if name == "crate_log" {
                        println!("{}: {}", func, data.unwrap());
                        return arma_rs::Result::<(), ()>::Continue;
                    }
                    assert_eq!(name, "crate_server");
                    assert_eq!(func, "needs_link");
                    arma_rs::Result::Ok(())
                },
                Duration::from_secs(10)
            )
            .is_ok());
    }
}
