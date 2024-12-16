use std::collections::HashMap;

use serenity::model::prelude::UserId;
use sqlx::{Executor, Postgres};
use synixe_meta::discord::BRODSKY;
use synixe_model::gear::Deposit;
use uuid::Uuid;

pub async fn balance<'a, E>(member: &UserId, executor: E) -> Result<Option<i32>, anyhow::Error>
where
    E: Executor<'a, Database = Postgres>,
{
    let query = sqlx::query!(
        "SELECT balance FROM gear_bank_balance_cache WHERE member = $1 LIMIT 1",
        if member == &BRODSKY {
            "0".to_string()
        } else {
            member.to_string()
        },
    );
    query
        .fetch_optional(executor)
        .await?
        .map(|row| Ok(row.balance))
        .transpose()
}

pub async fn deposit(
    member: &UserId,
    amount: i32,
    reason: &str,
    id: Option<Uuid>,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        "INSERT INTO gear_bank_deposits (member, amount, reason, id) VALUES ($1, $2, $3, $4)",
        member.to_string(),
        amount,
        reason,
        id.unwrap_or_else(Uuid::new_v4),
    );
    query.execute(&mut **executor).await?;
    Ok(())
}

pub async fn deposit_search(
    member: &UserId,
    id: Option<Uuid>,
    reason: Option<String>,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<Deposit>, anyhow::Error> {
    let query = sqlx::query_as!(
        Deposit,
        "SELECT member, amount, reason, id, created FROM gear_bank_deposits WHERE member = $1",
        member.to_string(),
    );
    let res = query.fetch_all(&mut **executor).await?;
    Ok(res
        .into_iter()
        .filter(|row| id.map_or(true, |id| row.id() == id))
        .filter(|row| reason.clone().map_or(true, |reason| row.reason() == reason))
        .collect())
}

pub async fn transfer(
    source: &UserId,
    target: &UserId,
    amount: i32,
    reason: &str,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        "INSERT INTO gear_bank_transfers (source, target, amount, reason) VALUES ($1, $2, $3, $4)",
        source.to_string(),
        target.to_string(),
        amount,
        reason,
    );
    query.execute(&mut **executor).await?;
    Ok(())
}

pub async fn shop_purchase(
    member: &UserId,
    items: &HashMap<String, i32>,
    reason: &str,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), anyhow::Error> {
    for (class, quantity) in items {
        let query = sqlx::query!(
            "INSERT INTO gear_bank_purchases
                    (member, class, quantity, reason, personal, company)
                    SELECT $1, $2, $3, $4, cost.personal_current, cost.company_current
                    FROM gear_item_current_cost($2) AS cost",
            member.to_string(),
            class,
            quantity,
            reason,
        );
        query.execute(&mut **executor).await?;
    }
    Ok(())
}

pub async fn shop_purchase_personal_cost(
    member: &UserId,
    items: &HashMap<String, (i32, i32)>,
    reason: &str,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), anyhow::Error> {
    for (class, (quantity, cost)) in items {
        let query = sqlx::query!(
            "INSERT INTO gear_bank_purchases (member, class, quantity, personal, reason) VALUES ($1, $2, $3, $4, $5)",
            member.to_string(),
            class,
            quantity,
            cost,
            reason,
        );
        query.execute(&mut **executor).await?;
    }
    Ok(())
}
