use async_trait::async_trait;
use synixe_events::{
    gear::db::{Request, Response},
    respond,
};

use crate::actor;

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::LoadoutGet { member } => {
                match_with_return!(actor::gear::loadout::get(member, &*db), LoadoutGet, msg, cx)
            }
            Self::LoadoutStore { member, loadout } => {
                quick_transaction!(
                    LoadoutStore,
                    db,
                    msg,
                    cx,
                    actor::gear::loadout::store,
                    member,
                    loadout,
                )
            }
            Self::LockerGet { member } => {
                match_with_return!(actor::gear::locker::get(member, &*db), LockerGet, msg, cx)
            }
            Self::LockerStore { member, items } => {
                quick_transaction!(
                    LockerStore,
                    db,
                    msg,
                    cx,
                    actor::gear::locker::store,
                    member,
                    items,
                )
            }
            Self::LockerTake { member, items } => {
                quick_transaction!(
                    LockerTake,
                    db,
                    msg,
                    cx,
                    actor::gear::locker::take,
                    member,
                    items,
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
                quick_transaction!(
                    BankDepositNew,
                    db,
                    msg,
                    cx,
                    actor::gear::bank::deposit,
                    member,
                    *amount,
                    reason,
                    *id,
                )
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
                quick_transaction!(
                    BankTransferNew,
                    db,
                    msg,
                    cx,
                    actor::gear::bank::transfer,
                    source,
                    target,
                    *amount,
                    reason,
                )
            }
            Self::BankPurchasesNew { member, items } => {
                quick_transaction!(
                    BankPurchasesNew,
                    db,
                    msg,
                    cx,
                    actor::gear::bank::purchase,
                    member,
                    items,
                )
            }
            Self::ShopGetAll {} => {
                quick_transaction_return!(ShopGetAll, db, msg, cx, actor::gear::shop::items,)
            }
            Self::ShopGetPrice { item } => {
                quick_transaction_return!(ShopGetPrice, db, msg, cx, actor::gear::shop::price, item,)
            }
            Self::ShopEnter { member, items } => {
                let mut tx = transaction!(db, msg, cx);
                // Store a blank loadout
                actor::gear::loadout::store(
                    member,
                    r#"[[],[],[],[],[],[],"","",[],["","","","","",""]]"#,
                    &mut tx,
                )
                .await?;
                // Store the items
                actor::gear::locker::store(member, items, &mut tx).await?;
                // Fetch the player's balance
                let Some(balance) = actor::gear::bank::balance(member, &mut tx).await? else {
                    respond!(msg, Response::ShopEnter(Err("No balance found".into()))).await?;
                    return Err(anyhow::anyhow!("No balance found"));
                };
                // Fetch the player's locker
                let locker = actor::gear::locker::get(member, &mut tx).await?;
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
                actor::gear::locker::take(member, items, &mut tx).await?;
                // Store the loadout
                actor::gear::loadout::store(member, loadout, &mut tx).await?;
                tx.commit().await?;
                respond!(msg, Response::ShopLeave(Ok(()))).await?;
                Ok(())
            }
            Self::ShopPurchase { member, items } => {
                let mut tx = transaction!(db, msg, cx);
                // Take the items from the locker
                actor::gear::bank::shop_purchase(member, items, &mut tx).await?;
                // Fetch the player's balance
                let Some(balance) = actor::gear::bank::balance(member, &mut tx).await? else {
                    respond!(msg, Response::ShopPurchase(Err("No balance found".into()))).await?;
                    return Err(anyhow::anyhow!("No balance found"));
                };
                // Fetch the player's locker
                let locker = actor::gear::locker::get(member, &mut tx).await?;
                tx.commit().await?;
                respond!(msg, Response::ShopPurchase(Ok((locker, balance)))).await?;
                Ok(())
            }
        }
    }
}
