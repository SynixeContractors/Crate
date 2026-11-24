use std::collections::HashMap;

use serenity::model::prelude::UserId;
use sqlx::{Executor, Postgres};
use synixe_events::{
    gear::db::{
        BankRequest, BankRequestStatus, CompanyActivity, PurchasedItem, Transaction,
        TransactionKind, Transfer,
    },
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
        if *bullets_needed <= 0 || *bullets_per_magazine <= 0 {
            continue;
        }
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
            let partial_cost_factor =
                f64::from(remainder_bullets) / f64::from(*bullets_per_magazine);
            sqlx::query!(
                "INSERT INTO gear_bank_purchases (member, class, quantity, reason, personal, company, created, add_item)
                 SELECT $1, $2, $3, $4, CEIL(cost.personal_current * $5), CEIL(cost.company_current * $5), NOW() + INTERVAL '2 millisecond', false
                 FROM gear_item_current_cost($2) AS cost",
                member.to_string(),
                class,
                1,
                format!("shop refill {remainder_bullets} bullets for {class}"),
                // Overrides the type sqlx infers for $5 from the integer cost it multiplies
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
        .map(|row| {
            (
                row.category,
                i32::try_from(row.cost.unwrap_or(0)).unwrap_or(0),
            )
        })
        .filter(|(_, cost)| *cost != 0)
        .filter_map(|(category, cost)| category.map(|category| (category, cost)))
        .collect())
}

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
    // Every column a UNION branch reads is NOT NULL, except the item columns only purchases have
    let rows = sqlx::query_as!(
        LedgerRow,
        r#"SELECT amount AS "amount!", created AS "created!", kind AS "kind!", reason AS "reason!", class AS "class?", pretty AS "pretty?", quantity AS "quantity?"
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
ORDER BY created DESC;"#,
        member.to_string(),
    )
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

pub async fn transfers(
    member: &UserId,
    limit: i64,
    offset: i64,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<Transfer>, anyhow::Error> {
    let rows = sqlx::query!(
        "SELECT source, target, amount, reason, created
        FROM gear_bank_transfers
        WHERE source = $1 OR target = $1
        ORDER BY created DESC
        LIMIT $2 OFFSET $3",
        member.to_string(),
        limit,
        offset,
    )
    .fetch_all(&mut **executor)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(Transfer {
                source: stored_member(&row.source)?,
                target: stored_member(&row.target)?,
                amount: row.amount,
                reason: row.reason,
                created: row.created,
            })
        })
        .collect()
}

/// Parses a member ID as stored in the bank tables, where Brodsky (the company account) is "0"
fn stored_member(id: &str) -> Result<UserId, anyhow::Error> {
    match id.parse::<u64>()? {
        0 => Ok(BRODSKY),
        id => Ok(UserId::new(id)),
    }
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

/// How the company has been doing over the last `days`. Spending is counted the way the company balance is
/// (see `gear_bank_balance_cache`): its own withdrawals, everything paid to members, garage purchases and the
/// company's share of gear purchases. Missions are counted by when they were played.
pub async fn company_activity<'a, E>(
    days: i32,
    executor: E,
) -> Result<CompanyActivity, anyhow::Error>
where
    E: Executor<'a, Database = Postgres>,
{
    let row = sqlx::query!(
        r#"SELECT
            (SELECT balance FROM gear_bank_balance_cache WHERE member = '0') AS balance,
            (SELECT COALESCE(SUM(amount), 0) FROM gear_bank_deposits
                WHERE member = '0' AND amount > 0 AND created > NOW() - make_interval(days => $1)) AS "income!",
            (SELECT COALESCE(-SUM(amount) FILTER (WHERE member = '0' AND amount < 0), 0)
                + COALESCE(SUM(amount) FILTER (WHERE member <> '0'), 0)
                FROM gear_bank_deposits WHERE created > NOW() - make_interval(days => $1))
            + (SELECT COALESCE(SUM(cost), 0) FROM garage_purchases WHERE created > NOW() - make_interval(days => $1))
            + (SELECT COALESCE(SUM(company * quantity), 0) FROM gear_bank_purchases
                WHERE created > NOW() - make_interval(days => $1)) AS "spending!",
            (SELECT COALESCE(SUM(reputation), 0) FROM reputation_events
                WHERE created > NOW() - make_interval(days => $1)) AS "reputation!",
            (SELECT COUNT(*) FILTER (WHERE outcome = 'success') FROM missions_schedule
                WHERE start > NOW() - make_interval(days => $1)) AS "successes!",
            (SELECT COUNT(*) FILTER (WHERE outcome = 'partial') FROM missions_schedule
                WHERE start > NOW() - make_interval(days => $1)) AS "partials!",
            (SELECT COUNT(*) FILTER (WHERE outcome = 'failure') FROM missions_schedule
                WHERE start > NOW() - make_interval(days => $1)) AS "failures!""#,
        days,
    )
    .fetch_one(executor)
    .await?;
    Ok(CompanyActivity {
        balance: row.balance.map_or(0, i64::from),
        income: row.income,
        spending: row.spending,
        reputation: row.reputation,
        successes: row.successes,
        partials: row.partials,
        failures: row.failures,
    })
}

/// The average price members paid for an item each week over the last year, oldest first. Partial
/// magazine refills are left out, as they're charged for only the bullets topped up.
pub async fn price_history<'a, E>(
    item: &str,
    executor: E,
) -> Result<Vec<(time::OffsetDateTime, i32)>, anyhow::Error>
where
    E: Executor<'a, Database = Postgres>,
{
    let rows = sqlx::query!(
        r#"SELECT date_trunc('week', created) AS "week!", ROUND(AVG(personal))::INT AS "price!"
        FROM gear_bank_purchases
        WHERE class = $1
            AND quantity > 0
            AND personal > 0
            AND reason NOT LIKE 'shop refill % bullets for %'
            AND created > NOW() - INTERVAL '1 year'
        GROUP BY 1
        ORDER BY 1"#,
        item,
    )
    .fetch_all(executor)
    .await?;
    Ok(rows.into_iter().map(|row| (row.week, row.price)).collect())
}

#[derive(sqlx::FromRow)]
struct RequestRow {
    id: uuid::Uuid,
    requester: String,
    payer: String,
    amount: i32,
    reason: String,
    status: String,
    created: time::OffsetDateTime,
    resolved: Option<time::OffsetDateTime>,
}

impl TryFrom<RequestRow> for BankRequest {
    type Error = anyhow::Error;

    fn try_from(row: RequestRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            requester: stored_member(&row.requester)?,
            payer: stored_member(&row.payer)?,
            amount: row.amount,
            reason: row.reason,
            status: BankRequestStatus::parse(&row.status)
                .ok_or_else(|| anyhow::anyhow!("unknown bank request status: {}", row.status))?,
            created: row.created,
            resolved: row.resolved,
        })
    }
}

pub async fn create_request(
    requester: &UserId,
    payer: &UserId,
    amount: i32,
    reason: &str,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<uuid::Uuid, anyhow::Error> {
    let row = sqlx::query!(
        "INSERT INTO bank_requests (requester, payer, amount, reason) VALUES ($1, $2, $3, $4) RETURNING id",
        requester.to_string(),
        payer.to_string(),
        amount,
        reason,
    )
    .fetch_one(&mut **executor)
    .await?;
    Ok(row.id)
}

/// Moves a request from `from` to `to`, if it's in `from` and `payer` is the one asked. `None` when it isn't.
pub async fn transition_request(
    id: uuid::Uuid,
    payer: &UserId,
    from: BankRequestStatus,
    to: BankRequestStatus,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Option<BankRequest>, anyhow::Error> {
    // Only a request leaving "paying" or "open" is timestamped, so `resolved` is when it was finished
    let row = sqlx::query_as!(
        RequestRow,
        r#"UPDATE bank_requests
        SET status = $4,
            resolved = CASE WHEN $4::VARCHAR IN ('paid', 'declined') THEN NOW() ELSE NULL END
        WHERE id = $1 AND payer = $2 AND status = $3
        RETURNING id, requester, payer, amount, reason, status, created, resolved"#,
        id,
        payer.to_string(),
        from.as_str(),
        to.as_str(),
    )
    .fetch_optional(&mut **executor)
    .await?;
    row.map(BankRequest::try_from).transpose()
}

/// A member's requests, as requester or payer, newest first
pub async fn requests(
    member: &UserId,
    limit: i64,
    offset: i64,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<BankRequest>, anyhow::Error> {
    let rows = sqlx::query_as!(
        RequestRow,
        r#"SELECT id, requester, payer, amount, reason, status, created, resolved
        FROM bank_requests
        WHERE requester = $1 OR payer = $1
        ORDER BY created DESC
        LIMIT $2 OFFSET $3"#,
        member.to_string(),
        limit,
        offset,
    )
    .fetch_all(&mut **executor)
    .await?;
    rows.into_iter().map(BankRequest::try_from).collect()
}
