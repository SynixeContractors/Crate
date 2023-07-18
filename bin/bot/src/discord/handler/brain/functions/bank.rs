use serde_json::json;
use serenity::{model::prelude::UserId, prelude::Context};
use synixe_events::gear::db::Response;
use synixe_meta::discord::BRODSKY;
use synixe_proc::events_request_2;

use super::BrainFunction;

pub struct GetBalance {}

#[async_trait::async_trait]
impl BrainFunction for GetBalance {
    fn name(&self) -> &'static str {
        "bank_get_balance"
    }

    fn desc(&self) -> &'static str {
        "Get's the balance of a member or the company, use names_lookup_members to get the member id. use 1028418063168708638 for company balance"
    }

    fn args(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "members": {
                    "type": "array",
                    "description": "member ids to get the balance of, brodsky's id (1028418063168708638) returns the company's balance",
                    "items": {
                        "type": "string",
                    },
                }
            }
        })
    }

    async fn run(&self, _ctx: &Context, args: serde_json::Value) -> Option<serde_json::Value> {
        let members = args["members"]
            .as_array()?
            .iter()
            .map(|v| {
                let id = v.as_str().unwrap_or_default();
                let mut member = if id.trim().is_empty() {
                    UserId(0)
                } else {
                    UserId(id.parse().expect("invalid id"))
                };
                if member == BRODSKY {
                    member = UserId(0);
                }
                member
            })
            .collect::<Vec<_>>();

        let mut responses = Vec::new();

        for member in members {
            let Ok(Ok((Response::BankBalance(Ok(Some(balance))), _))) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                BankBalance {
                    member,
                }
            )
            .await else {
                return None;
            };

            if member == UserId(0) {
                return Some(json!({
                    "bank": bootstrap::format::money(balance, false),
                }));
            }

            let Ok(Ok((Response::LockerBalance(Ok(locker_balance)), _))) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                LockerBalance {
                    member,
                }
            )
            .await else {
                return None;
            };
            let  Ok(Ok((Response::LoadoutBalance(Ok(loadout_balance)), _))) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                LoadoutBalance {
                    member,
                }
            )
            .await else {
                return None;
            };

            responses.push(json!({
                member.to_string(): {
                    "bank": bootstrap::format::money(balance, false),
                    "locker": bootstrap::format::money(locker_balance, false),
                    "loadout": bootstrap::format::money(loadout_balance, false),
            }}));
        }
        Some(json!(responses))
    }
}
