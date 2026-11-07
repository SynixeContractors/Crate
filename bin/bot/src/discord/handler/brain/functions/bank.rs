use serde_json::json;
use serenity::{all::UserId, prelude::Context};
use synixe_events::gear::db::Response;
use synixe_meta::discord::BRODSKY;
use synixe_proc::events_request_5;

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
            let Ok(Ok((Response::BankBalance(Ok(Some(balance))), _))) = events_request_5!(
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

            let Ok(Ok((Response::LockerBalance(Ok(locker_balance)), _))) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                LockerBalance { member }
            )
            .await
            else {
                return None;
            };
            let Ok(Ok((Response::LoadoutBalance(Ok(loadout_balance)), _))) = events_request_5!(
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

async fn resolve_member(ctx: &Context, args: &serde_json::Value) -> Result<UserId, String> {
    let name = args["name"]
        .as_str()
        .map(std::string::ToString::to_string)
        .or_else(|| args["name"].as_u64().map(|v| v.to_string()))
        .ok_or_else(|| "missing name".to_string())?;
    let (found, _) = find_members(ctx, std::slice::from_ref(&name)).await?;
    found
        .into_iter()
        .next()
        .map(|(_, member)| member)
        .ok_or_else(|| format!("no member found matching {name}"))
}

pub struct Spending {}

#[async_trait::async_trait]
impl BrainFunction for Spending {
    fn name(&self) -> &'static str {
        "bank_spending"
    }

    fn desc(&self) -> &'static str {
        "Get how much a member has spent from their bank account, broken down by category"
    }

    fn args(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "discord id or name of the member",
                },
            },
            "required": ["name"],
        })
    }

    async fn run(&self, ctx: &Context, args: serde_json::Value) -> Option<serde_json::Value> {
        let member = match resolve_member(ctx, &args).await {
            Ok(member) => member,
            Err(error) => return Some(json!({ "error": error })),
        };
        let Ok(Ok((Response::BankSpent(Ok(spent)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            BankSpent { member }
        )
        .await
        else {
            return None;
        };
        Some(json!(
            spent
                .into_iter()
                .map(|(category, amount)| (category, bootstrap::format::money(amount, false)))
                .collect::<std::collections::HashMap<_, _>>()
        ))
    }
}

pub struct History {}

#[async_trait::async_trait]
impl BrainFunction for History {
    fn name(&self) -> &'static str {
        "bank_history"
    }

    fn desc(&self) -> &'static str {
        "Get the most recent bank transactions for a member, newest first"
    }

    fn args(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "discord id or name of the member",
                },
            },
            "required": ["name"],
        })
    }

    async fn run(&self, ctx: &Context, args: serde_json::Value) -> Option<serde_json::Value> {
        let member = match resolve_member(ctx, &args).await {
            Ok(member) => member,
            Err(error) => return Some(json!({ "error": error })),
        };
        let Ok(Ok((Response::BankHistory(Ok(mut history)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            BankHistory { member }
        )
        .await
        else {
            return None;
        };
        history.sort_by_key(|t| std::cmp::Reverse(t.created));
        Some(json!(
            history
                .iter()
                .take(10)
                .map(|t| json!({
                    "date": t.created.date().to_string(),
                    "amount": bootstrap::format::money(t.amount, false),
                }))
                .collect::<Vec<_>>()
        ))
    }
}

pub struct ShopPrice {}

#[async_trait::async_trait]
impl BrainFunction for ShopPrice {
    fn name(&self) -> &'static str {
        "shop_price"
    }

    fn desc(&self) -> &'static str {
        "Get the price of an item in the shop, for a personal purchase and a company purchase"
    }

    fn args(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "item": {
                    "type": "string",
                    "description": "class name of the item in the shop",
                },
            },
            "required": ["item"],
        })
    }

    async fn run(&self, _ctx: &Context, args: serde_json::Value) -> Option<serde_json::Value> {
        let item = args["item"].as_str()?.to_string();
        let Ok(Ok((Response::ShopGetPrice(Ok(price)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            ShopGetPrice { item }
        )
        .await
        else {
            return None;
        };
        Some(json!({
            "personal": bootstrap::format::money(price.personal(), false),
            "company": bootstrap::format::money(price.company(), false),
            "sale_ends": price.until().map(|until| until.date().to_string()),
        }))
    }
}
