use async_trait::async_trait;
use opentelemetry::trace::{FutureExt, Span};
use synixe_events::{
    gear::db::{Request, Response},
    respond,
};
use synixe_model::gear::Price;
use uuid::Uuid;

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
        cx: opentelemetry::Context,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::LoadoutGet { member } => fetch_one_and_respond!(
                msg,
                *db,
                cx,
                Response::LoadoutGet,
                "SELECT loadout as value FROM gear_loadouts WHERE member = $1",
                member.0.to_string(),
            ),
            Self::LoadoutSet { member, loadout } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::LoadoutSet,
                    "INSERT INTO gear_loadouts (member, loadout) VALUES ($1, $2) ON CONFLICT (member) DO UPDATE SET loadout = $2",
                    member.0.to_string(),
                    loadout,
                )
            }
            Self::LockerGet { member } => {
                let (query, mut span) = trace_query!(
                    cx,
                    "SELECT class, quantity FROM gear_locker WHERE member = $1",
                    member.0.to_string(),
                );
                let res = query.fetch_all(&*db).await;
                span.end();
                match res {
                    Ok(res) => {
                        let res = res
                            .into_iter()
                            .map(|row| (row.class, row.quantity))
                            .collect();
                        if let Err(e) = respond!(msg, Response::LockerGet(Ok(res)))
                            .with_context(cx)
                            .await
                        {
                            error!("Failed to respond to LockerGet: {}", e);
                        }
                    }
                    Err(e) => {
                        if let Err(e) = respond!(msg, Response::LockerGet(Err(e.to_string())))
                            .with_context(cx)
                            .await
                        {
                            error!("Failed to respond to LockerGet: {}", e);
                        }
                    }
                }
                Ok(())
            }
            Self::LockerStore { member, items } => {
                match db.begin().await {
                    Ok(mut tx) => {
                        for item in items {
                            let (query, mut span) = trace_query!(
                                cx,
                                "INSERT INTO gear_locker_log (member, class, quantity) VALUES ($1, $2, $3)",
                                member.0.to_string(),
                                item.0,
                                item.1,
                            );
                            let res = query.execute(&mut tx).await;
                            span.end();
                            if let Err(e) = res {
                                if let Err(e) =
                                    respond!(msg, Response::LockerStore(Err(e.to_string())))
                                        .with_context(cx)
                                        .await
                                {
                                    error!("Failed to respond to LockerStore: {}", e);
                                }
                                return Ok(());
                            }
                        }
                        if let Err(e) = respond!(msg, Response::LockerStore(Ok(())))
                            .with_context(cx)
                            .await
                        {
                            error!("Failed to respond to LockerStore: {}", e);
                        }
                    }
                    Err(e) => {
                        if let Err(e) = respond!(msg, Response::LockerStore(Err(e.to_string())))
                            .with_context(cx)
                            .await
                        {
                            error!("Failed to respond to LockerStore: {}", e);
                        }
                    }
                }
                Ok(())
            }
            Self::LockerTake { member, items } => {
                match db.begin().await {
                    Ok(mut tx) => {
                        for item in items {
                            let (query, mut span) = trace_query!(
                                cx,
                                "INSERT INTO gear_locker_log (member, class, quantity) VALUES ($1, $2, $3)",
                                member.0.to_string(),
                                item.0,
                                -item.1,
                            );
                            let res = query.execute(&mut tx).await;
                            span.end();
                            if let Err(e) = res {
                                if let Err(e) =
                                    respond!(msg, Response::LockerTake(Err(e.to_string())))
                                        .with_context(cx)
                                        .await
                                {
                                    error!("Failed to respond to LockerTake: {}", e);
                                }
                                return Ok(());
                            }
                        }
                        if let Err(e) = respond!(msg, Response::LockerTake(Ok(())))
                            .with_context(cx)
                            .await
                        {
                            error!("Failed to respond to LockerTake: {}", e);
                        }
                    }
                    Err(e) => {
                        if let Err(e) = respond!(msg, Response::LockerTake(Err(e.to_string())))
                            .with_context(cx)
                            .await
                        {
                            error!("Failed to respond to LockerTake: {}", e);
                        }
                    }
                }
                Ok(())
            }
            Self::BankBalance { member } => fetch_one_and_respond!(
                msg,
                *db,
                cx,
                Response::BankBalance,
                "SELECT balance as value FROM gear_bank_balance_cache WHERE member = $1",
                member.0.to_string(),
            ),
            Self::BankDepositNew {
                member,
                amount,
                reason,
                id,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::BankDepositNew,
                    "INSERT INTO gear_bank_deposits (member, amount, reason, id) VALUES ($1, $2, $3, $4)",
                    member.0.to_string(),
                    amount,
                    reason,
                    id.unwrap_or_else(Uuid::new_v4),
                )
            }
            Self::BankTransferNew {
                source,
                target,
                amount,
                reason,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::BankTransferNew,
                    "INSERT INTO gear_bank_transfers (source, target, amount, reason) VALUES ($1, $2, $3, $4)",
                    source.0.to_string(),
                    target.0.to_string(),
                    amount,
                    reason,
                )
            }
            Self::BankPurchasesNew { member, items } => {
                match db.begin().await {
                    Ok(mut tx) => {
                        for item in items {
                            let (query, mut span) = trace_query!(
                                cx,
                                "INSERT INTO gear_bank_purchases (member, class, quantity, global, cost) VALUES ($1, $2, $3, $4, (SELECT cost FROM gear_item_current_cost($2)))",
                                member.0.to_string(),
                                item.0,
                                item.1,
                                item.2,
                            );
                            let res = query.execute(&mut tx).await;
                            span.end();
                            if let Err(e) = res {
                                if let Err(e) =
                                    respond!(msg, Response::BankPurchasesNew(Err(e.to_string())))
                                        .with_context(cx)
                                        .await
                                {
                                    error!("Failed to respond to BankPurchasesNew: {}", e);
                                }
                                return Ok(());
                            }
                        }
                        if let Err(e) = respond!(msg, Response::BankPurchasesNew(Ok(())))
                            .with_context(cx)
                            .await
                        {
                            error!("Failed to respond to BankPurchasesNew: {}", e);
                        }
                    }
                    Err(e) => {
                        if let Err(e) =
                            respond!(msg, Response::BankPurchasesNew(Err(e.to_string())))
                                .with_context(cx)
                                .await
                        {
                            error!("Failed to respond to BankPurchasesNew: {}", e);
                        }
                    }
                }
                Ok(())
            }
            Self::ShopGetAll {} => {
                let (query, mut span) = trace_query!(cx,
                    "SELECT i.class, i.global, gear_item_base_cost(i.class) as base, c.cost, c.end_date FROM gear_items i, LATERAL gear_item_current_cost(i.class) c",
                );
                let res = query.fetch_all(&*db).await;
                span.end();
                match res {
                    Ok(res) => {
                        let res = res
                            .into_iter()
                            .filter(|row| row.base.is_some())
                            .map(|row| {
                                (
                                    row.class,
                                    Price::new(
                                        row.base.unwrap(),
                                        row.cost,
                                        row.end_date,
                                        row.global,
                                    ),
                                )
                            })
                            .collect();
                        if let Err(e) = respond!(msg, Response::ShopGetAll(Ok(res)))
                            .with_context(cx)
                            .await
                        {
                            error!("Failed to respond to ShopGetAll: {}", e);
                        }
                    }
                    Err(e) => {
                        if let Err(e) = respond!(msg, Response::ShopGetAll(Err(e.to_string())))
                            .with_context(cx)
                            .await
                        {
                            error!("Failed to respond to ShopGetAll: {}", e);
                        }
                    }
                }
                Ok(())
            }
            Self::ShopGetPrice { item } => {
                let (query, mut span) = trace_query!(cx,
                    "SELECT i.global, gear_item_base_cost(i.class) as base, c.cost, c.end_date FROM gear_items i, LATERAL gear_item_current_cost(i.class) c WHERE i.class = $1",
                    item,
                );
                let res = query.fetch_one(&*db).await;
                span.end();
                match res {
                    Ok(row) => {
                        if let Err(e) = respond!(
                            msg,
                            Response::ShopGetPrice(Ok(Price::new(
                                row.base.unwrap_or(-1),
                                row.cost,
                                row.end_date,
                                row.global
                            )))
                        )
                        .with_context(cx)
                        .await
                        {
                            error!("Failed to respond to ShopGetPrice: {}", e);
                        }
                    }
                    Err(e) => {
                        if let Err(e) = respond!(msg, Response::ShopGetPrice(Err(e.to_string())))
                            .with_context(cx)
                            .await
                        {
                            error!("Failed to respond to ShopGetPrice: {}", e);
                        }
                    }
                }
                Ok(())
            }
        }
    }
}
