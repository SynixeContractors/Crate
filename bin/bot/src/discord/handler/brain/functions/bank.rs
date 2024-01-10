use serde_json::json;
use serenity::{model::prelude::UserId, prelude::Context};
use synixe_events::gear::db::Response;
use synixe_meta::discord::BRODSKY;
use synixe_proc::events_request_2;

use crate::discord::utils::find_members;

use super::BrainFunction;

pub struct GetBalance {}

#[async_trait::async_trait]
impl BrainFunction for GetBalance {
    fn name(&self) -> &'static str {
        "bank_get_balance"
    }

    fn desc(&self) -> &'static str {
        "Get's the balance of a member or the company. use 'Ctirad Brodsky' for company balance"
    }

    fn args(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "names": {
                    "type": "array",
                    "description": "names to get the balance of, 'Ctirad Brodsky' returns the company's balance",
                    "items": {
                        "type": "string",
                        "description": "discord id or name of the member to get the balance of"
                    },
                }
            }
        })
    }

    async fn run(&self, ctx: &Context, args: serde_json::Value) -> Option<serde_json::Value> {
        let names = args["names"]
            .as_array()?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(std::string::ToString::to_string)
                    .or_else(|| v.as_u64().map(|v| v.to_string()))
                    .expect("name is string or u64")
            })
            .collect::<Vec<_>>();
        let (found, _) = match find_members(ctx, &names).await {
            Ok(members) => members,
            Err(e) => {
                error!("failed to find members: {}", e);
                return Some(json!({
                    "error": e,
                }));
            }
        };

        info!("found members: {:?}", found);

        let mut responses = Vec::new();

        for (_, member) in found {
            let Ok(Ok((Response::BankBalance(Ok(Some(balance))), _))) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                BankBalance { member }
            )
            .await
            else {
                return None;
            };

            if member == BRODSKY {
                responses.push(json!({
                    "company": bootstrap::format::money(balance, false),
                }));
            }

            let Ok(Ok((Response::LockerBalance(Ok(locker_balance)), _))) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                LockerBalance { member }
            )
            .await
            else {
                return None;
            };
            let Ok(Ok((Response::LoadoutBalance(Ok(loadout_balance)), _))) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                LoadoutBalance { member }
            )
            .await
            else {
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
