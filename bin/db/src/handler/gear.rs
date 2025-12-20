use std::collections::HashMap;

use arma_rs::{FromArma, loadout::Loadout};
use async_trait::async_trait;
use synixe_events::{
    gear::db::{Request, Response},
    respond,
};
use synixe_model::gear::FamilyItem;

use crate::{
    actor::{self, gear::fuel::prices_for_map},
    game_audit,
};

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::LoadoutGet { member, campaign } => {
                match_with_return!(
                    actor::gear::loadout::get(member, campaign, &*db),
                    LoadoutGet,
                    msg,
                    cx
                )
            }
            Self::LoadoutStore {
                member,
                campaign,
                loadout,
            } => {
                quick_transaction!(
                    LoadoutStore,
                    db,
                    msg,
                    cx,
                    actor::gear::loadout::store,
                    member,
                    campaign,
                    loadout,
                )
            }
            Self::LockerGet { member } => {
                match_with_return!(actor::gear::locker::get(member, &*db), LockerGet, msg, cx)
            }
            Self::LoadoutBalance { member } => {
                let Ok(Some(l)) = actor::gear::loadout::get(member, &None, &*db).await else {
                    return respond!(msg, Response::LoadoutBalance(Ok(0)))
                        .await
                        .map_err(Into::into);
                };
                let loadout_items = Loadout::from_arma(l)
                    .expect("should be valid loadout")
                    .classes();
                match_with_return!(
                    actor::gear::loadout::balance(loadout_items, &*db),
                    LoadoutBalance,
                    msg,
                    cx
                )
            }
            Self::LockerBalance { member } => {
                match_with_return!(
                    actor::gear::locker::balance(member, &*db),
                    LockerBalance,
                    msg,
                    cx
                )
            }
            Self::LockerStore {
                member,
                items,
                reason,
            } => {
                quick_transaction!(
                    LockerStore,
                    db,
                    msg,
                    cx,
                    actor::gear::locker::store,
                    member,
                    items,
                    reason,
                )
            }
            Self::LockerTake {
                member,
                items,
                reason,
            } => {
                quick_transaction!(
                    LockerTake,
                    db,
                    msg,
                    cx,
                    actor::gear::locker::take,
                    member,
                    items,
                    reason,
                )
            }
            Self::BankBalance { member } => {
                match_with_return!(
                    actor::gear::bank::balance(member, &*db),
                    BankBalance,
                    msg,
                    cx
                )
            }
            Self::BankDepositNew {
                member,
                amount,
                reason,
                id,
            } => {
                let res = quick_transaction!(
                    BankDepositNew,
                    db,
                    msg,
                    cx,
                    actor::gear::bank::deposit,
                    member,
                    *amount,
                    reason,
                    *id,
                );
                let member = member.to_owned();
                let reason = reason.to_owned();
                tokio::spawn(async move {
                    actor::gear::bank::publish_balance(nats, member, reason).await;
                });
                res
            }
            Self::BankDepositSearch { member, id, reason } => {
                quick_transaction_return!(
                    BankDepositSearch,
                    db,
                    msg,
                    cx,
                    actor::gear::bank::deposit_search,
                    member,
                    *id,
                    reason.clone(),
                )
            }
            Self::BankTransferNew {
                source,
                target,
                amount,
                reason,
            } => {
                let res = quick_transaction!(
                    BankTransferNew,
                    db,
                    msg,
                    cx,
                    actor::gear::bank::transfer,
                    source,
                    target,
                    *amount,
                    reason,
                );
                let source = *source;
                let target = *target;
                let nats = nats.clone();
                tokio::spawn(async move {
                    actor::gear::bank::publish_balance(
                        nats.clone(),
                        source,
                        String::from("transfer source"),
                    )
                    .await;
                    actor::gear::bank::publish_balance(
                        nats,
                        target,
                        String::from("transfer target"),
                    )
                    .await;
                });
                res
            }
            Self::ShopGetAll { page } => {
                quick_transaction_return!(
                    ShopGetAll,
                    db,
                    msg,
                    cx,
                    actor::gear::shop::items,
                    i64::from(*page),
                )
            }
            Self::ShopGetPrice { item } => {
                quick_transaction_return!(ShopGetPrice, db, msg, cx, actor::gear::shop::price, item,)
            }
            Self::ShopEnter { member, items } => {
                let mut tx = transaction!(db, msg, cx);
                // Store a blank loadout
                actor::gear::loadout::store(
                    member,
                    &None,
                    r#"[[],[],[],[],[],[],"","",[],["","","","","",""]]"#,
                    &mut tx,
                )
                .await?;
                // Store the items
                actor::gear::locker::store(member, items, "shop enter", &mut tx).await?;
                // Fetch the player's balance
                let Some(balance) = actor::gear::bank::balance(member, &mut *tx).await? else {
                    respond!(msg, Response::ShopEnter(Err("No balance found".into()))).await?;
                    return Err(anyhow::anyhow!("No balance found"));
                };
                // Fetch the player's locker
                let locker = actor::gear::locker::get(member, &mut *tx).await?;
                tx.commit().await?;
                respond!(msg, Response::ShopEnter(Ok((locker, balance)))).await?;
                Ok(())
            }
            Self::ShopLeave {
                member,
                items,
                loadout,
            } => {
                let mut tx = transaction!(db, msg, cx);
                // Take the items from the locker
                actor::gear::locker::take(member, items, "shop leave", &mut tx).await?;
                // Store the loadout
                actor::gear::loadout::store(member, &None, loadout, &mut tx).await?;
                tx.commit().await?;
                respond!(msg, Response::ShopLeave(Ok(()))).await?;
                Ok(())
            }
            Self::ShopPurchase { member, items } => {
                let mut tx = transaction!(db, msg, cx);
                // Take the items from the locker
                actor::gear::bank::shop_purchase(member, items, "shop purchase", &mut tx).await?;
                // Fetch the player's balance
                let Some(balance) = actor::gear::bank::balance(member, &mut *tx).await? else {
                    respond!(msg, Response::ShopPurchase(Err("No balance found".into()))).await?;
                    return Err(anyhow::anyhow!("No balance found"));
                };
                // Fetch the player's locker
                let locker = actor::gear::locker::get(member, &mut *tx).await?;
                tx.commit().await?;
                respond!(msg, Response::ShopPurchase(Ok((locker, balance)))).await?;
                actor::gear::bank::publish_balance(nats, *member, String::from("shop purchase"))
                    .await;
                Ok(())
            }
            Self::SetPrettyName { item, pretty } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::SetPrettyName,
                    "UPDATE gear_items SET pretty = $2 WHERE class = $1",
                    item,
                    pretty,
                )
            }
            Self::GetPrettyName { item } => {
                fetch_one_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::GetPrettyName,
                    "SELECT pretty as value FROM gear_items WHERE class = $1",
                    item,
                )
            }
            Self::FamilySearch { item, relation } => {
                let query = sqlx::query!(
                    "SELECT family,class,(SELECT pretty FROM gear_items WHERE class = gear_items_family.class) as pretty FROM gear_items_family WHERE family = (SELECT family FROM gear_items_family WHERE class = $1 AND relation = $2)",
                    item,
                    relation,
                );
                match query.fetch_all(&*db).await {
                    Ok(res) => {
                        respond!(
                            msg,
                            Response::FamilySearch(Ok(res
                                .into_iter()
                                .map(|row| FamilyItem {
                                    family: item.clone(),
                                    class: row.class,
                                    pretty: row.pretty.unwrap_or_default(),
                                })
                                .collect()))
                        )
                        .await?;
                    }
                    Err(e) => {
                        respond!(msg, Response::FamilySearch(Err(e.to_string()))).await?;
                    }
                }
                Ok(())
            }
            Self::FamilyCompatibleItems { member, relation } => {
                let query = sqlx::query!(
                    "SELECT family,class,(SELECT pretty FROM gear_items WHERE class = gear_items_family.class) as pretty FROM gear_items_family WHERE relation = $2 AND class IN (SELECT class FROM gear_locker WHERE member = $1)",
                    member.to_string(),
                    relation,
                );
                match query.fetch_all(&*db).await {
                    Ok(res) => {
                        respond!(
                            msg,
                            Response::FamilyCompatibleItems(Ok(res
                                .into_iter()
                                .map(|row| FamilyItem {
                                    family: row.family,
                                    class: row.class,
                                    pretty: row.pretty.unwrap_or_default(),
                                })
                                .collect()))
                        )
                        .await?;
                    }
                    Err(e) => {
                        respond!(msg, Response::FamilyCompatibleItems(Err(e.to_string()))).await?;
                    }
                }
                Ok(())
            }
            Self::FamilyReplace {
                member,
                original,
                new,
                reason,
                cost,
            } => {
                let mut tx = transaction!(db, msg, cx);
                // Take the original item
                actor::gear::locker::take(
                    member,
                    &{
                        let mut map = HashMap::new();
                        map.insert(original.clone(), 1);
                        map
                    },
                    reason,
                    &mut tx,
                )
                .await?;
                // Purchase the new item
                actor::gear::bank::shop_purchase_personal_cost(
                    member,
                    &{
                        let mut map = HashMap::new();
                        map.insert(new.clone(), (1, *cost));
                        map
                    },
                    reason,
                    &mut tx,
                )
                .await?;
                tx.commit().await?;
                respond!(msg, Response::FamilyReplace(Ok(()))).await?;
                actor::gear::bank::publish_balance(
                    nats,
                    *member,
                    format!("family replace: {reason}"),
                )
                .await;
                Ok(())
            }
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            Self::Fuel {
                member,
                amount,
                fuel_type,
                plate,
                map,
            } => {
                let price_raw = prices_for_map(map) * (*amount as f64) * fuel_type.multiplier();
                let price = price_raw.ceil() as i32;
                if let Err(e) = sqlx::query!(
                    "INSERT INTO gear_bank_deposits (member, amount, reason, id) VALUES (0, $1, $2, '00000000-0000-0000-0000-000000000001')",
                    price,
                    format!(
                        "Fuel: {} {amount}l @ {:.2} by {member}{}",
                        fuel_type.as_str(),
                        prices_for_map(map) * fuel_type.multiplier(),
                        plate
                            .as_ref()
                            .map_or_else(String::new, |plate| format!(" on {plate}"))
                    ),
                )
                .execute(&*db)
                .await {
                    return Err(anyhow::Error::new(e).context("Failed to record fuel purchase"));
                }
                respond!(msg, Response::Fuel(Ok(price))).await?;
                let _ = game_audit(format!(
                    "**Fuel**\n{member} purchased {} {amount}l @ {:.2} (Total: ${}){}",
                    fuel_type.as_str(),
                    prices_for_map(map) * fuel_type.multiplier(),
                    price,
                    plate
                        .as_ref()
                        .map_or_else(String::new, |plate| format!(" on {plate}"))
                ))
                .await;
                Ok(())
            }
            Self::FuelPrice { map } => {
                let price = prices_for_map(map);
                respond!(msg, Response::FuelPrice(Ok(price))).await?;
                Ok(())
            }
        }
    }
}
