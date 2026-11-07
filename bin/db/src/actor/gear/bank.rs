use std::collections::HashMap;

use serenity::model::prelude::UserId;
use sqlx::{Executor, Postgres};
use synixe_events::{
    gear::db::{PurchasedItem, Transaction, TransactionKind},
    publish,
};
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
        .filter(|row| id.is_none_or(|id| row.id() == id))
        .filter(|row| reason.clone().is_none_or(|reason| row.reason() == reason))
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

pub async fn shop_purchase_ammo_refill(
    member: &UserId,
    magazines: &HashMap<String, (i32, i32)>,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), anyhow::Error> {
    for (class, (bullets_needed, bullets_per_magazine)) in magazines {
        let full_mags = bullets_needed / bullets_per_magazine;
        let remainder_bullets = bullets_needed % bullets_per_magazine;

        if full_mags > 0 {
            sqlx::query!(
                "INSERT INTO gear_bank_purchases (member, class, quantity, reason, personal, company, created, add_item)
                 SELECT $1, $2, $3, $4, cost.personal_current, cost.company_current, NOW() + INTERVAL '1 millisecond', false
                 FROM gear_item_current_cost($2) AS cost",
                member.to_string(),
                class,
                full_mags,
                format!("shop refill {full_mags} full magazines for {class}"),
            )
            .execute(&mut **executor)
            .await?;
        }

        if remainder_bullets > 0 {
            let partial_cost_factor = remainder_bullets as f32 / *bullets_per_magazine as f32;
            sqlx::query!(
                "INSERT INTO gear_bank_purchases (member, class, quantity, reason, personal, company, created, add_item)
                 SELECT $1, $2, $3, $4, CEIL(cost.personal_current * $5), CEIL(cost.company_current * $5), NOW() + INTERVAL '2 millisecond', false
                 FROM gear_item_current_cost($2) AS cost",
                member.to_string(),
                class,
                1,
                format!("shop refill {remainder_bullets} bullets for {class}"),
                partial_cost_factor as f64,
            )
            .execute(&mut **executor)
            .await?;
        }
    }
    Ok(())
}

pub async fn spent(
    member: &UserId,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<HashMap<String, i32>, anyhow::Error> {
    let query = sqlx::query!(
        "SELECT
            gi.category,
            SUM(gbp.personal) AS cost
        FROM gear_bank_purchases gbp
        JOIN gear_items gi
            ON gbp.class = gi.class
        WHERE gbp.member = $1
        GROUP BY gi.category;",
        member.to_string(),
    );
    let res = query.fetch_all(&mut **executor).await?;
    Ok(res
        .into_iter()
        .map(|row| (row.category, row.cost.unwrap_or(0) as i32))
        .filter(|(_, cost)| *cost != 0)
        .filter_map(|(category, cost)| category.map(|category| (category, cost)))
        .collect())
}

#[derive(sqlx::FromRow)]
struct LedgerRow {
    amount: i32,
    created: time::OffsetDateTime,
    kind: String,
    reason: String,
    class: Option<String>,
    pretty: Option<String>,
    quantity: Option<i32>,
}

pub async fn history(
    member: &UserId,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<Transaction>, anyhow::Error> {
    let rows = sqlx::query_as::<_, LedgerRow>(
        "SELECT amount, created, kind, reason, class, pretty, quantity
FROM (
    SELECT
        member,
        amount,
        created,
        'deposit' AS kind,
        reason,
        NULL::VARCHAR AS class,
        NULL::VARCHAR AS pretty,
        NULL::INTEGER AS quantity
    FROM gear_bank_deposits

    UNION ALL

    SELECT
        target AS member,
        amount,
        created,
        'transfer_in' AS kind,
        reason,
        NULL::VARCHAR AS class,
        NULL::VARCHAR AS pretty,
        NULL::INTEGER AS quantity
    FROM gear_bank_transfers

    UNION ALL

    SELECT
        source AS member,
        -amount AS amount,
        created,
        'transfer_out' AS kind,
        reason,
        NULL::VARCHAR AS class,
        NULL::VARCHAR AS pretty,
        NULL::INTEGER AS quantity
    FROM gear_bank_transfers

    UNION ALL

    SELECT
        gbp.member,
        (-gbp.personal * gbp.quantity) AS amount,
        gbp.created,
        'purchase' AS kind,
        gbp.reason,
        gbp.class,
        gi.pretty,
        gbp.quantity
    FROM gear_bank_purchases gbp
    LEFT JOIN gear_items gi ON gi.class = gbp.class
) ledger
WHERE member = $1
ORDER BY created DESC;",
    )
    .bind(member.to_string())
    .fetch_all(&mut **executor)
    .await?;

    let mut ledger: Vec<Transaction> = Vec::new();
    // Items bought in one checkout share a timestamp, so they are merged into one entry
    let mut checkouts: HashMap<time::OffsetDateTime, usize> = HashMap::new();
    for row in rows {
        let kind = match row.kind.as_str() {
            "deposit" => TransactionKind::Deposit,
            "transfer_in" => TransactionKind::TransferIn,
            "transfer_out" => TransactionKind::TransferOut,
            "purchase" => TransactionKind::Purchase,
            other => anyhow::bail!("unknown transaction kind: {other}"),
        };
        let item = match (row.class, row.quantity) {
            (Some(class), Some(quantity)) => Some(PurchasedItem {
                name: row.pretty.unwrap_or(class),
                quantity,
            }),
            _ => None,
        };
        if kind == TransactionKind::Purchase {
            if let Some(&index) = checkouts.get(&row.created) {
                let entry = &mut ledger[index];
                entry.amount += row.amount;
                entry.items.extend(item);
                continue;
            }
            checkouts.insert(row.created, ledger.len());
        }
        ledger.push(Transaction {
            amount: row.amount,
            created: row.created,
            kind,
            reason: row.reason,
            items: item.into_iter().collect(),
        });
    }
    Ok(ledger)
}

pub async fn publish_balance(nats: async_nats::Client, member: UserId, reason: String) {
    let db = bootstrap::DB::get().await;
    let mut tx = db
        .begin()
        .await
        .expect("should be able to begin transaction");
    let Some(new_balance) = balance(&member, &mut *tx)
        .await
        .expect("should be able to get balance")
    else {
        tx.commit()
            .await
            .expect("should be able to commit transaction");
        return;
    };
    if let Err(e) = publish!(
        nats,
        synixe_events::gear::publish::Publish::BalanceChanged {
            member,
            new_balance,
            reason,
        }
    )
    .await
    {
        tracing::error!(
            "Failed to publish balance change for member {}: {}",
            member,
            e
        );
    }
}
