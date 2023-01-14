use std::collections::HashMap;

use serenity::model::prelude::UserId;
use sqlx::{Executor, Postgres};

pub async fn get<'a, E>(member: &UserId, executor: E) -> Result<HashMap<String, i32>, anyhow::Error>
where
    E: Executor<'a, Database = Postgres>,
{
    let query = sqlx::query!(
        "SELECT class, quantity FROM gear_locker WHERE member = $1",
        member.0.to_string(),
    );
    Ok(query.fetch_all(executor).await.map(|rows| {
        rows.into_iter()
            .map(|row| {
                let class: String = row.class;
                let quantity: i32 = row.quantity;
                (class, quantity)
            })
            .collect()
    })?)
}

pub async fn store(
    member: &UserId,
    items: &HashMap<String, i32>,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), anyhow::Error> {
    for (class, quantity) in items {
        let query = sqlx::query!(
            "INSERT INTO gear_locker_log (member, class, quantity) VALUES ($1, $2, $3)",
            member.0.to_string(),
            class,
            quantity,
        );
        query.execute(&mut *executor).await?;
    }
    Ok(())
}

pub async fn take(
    member: &UserId,
    items: &HashMap<String, i32>,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), anyhow::Error> {
    for (class, quantity) in items {
        let query = sqlx::query!(
            "INSERT INTO gear_locker_log (member, class, quantity) VALUES ($1, $2, $3)",
            member.0.to_string(),
            class,
            -quantity,
        );
        query.execute(&mut *executor).await?;
    }
    Ok(())
}

pub async fn balance<'a, E>(member: &UserId, executor: E) -> Result<i32, anyhow::Error>
where
    E: Executor<'a, Database = Postgres>,
{
    let query = sqlx::query!(
        "SELECT SUM(gc.cost * gl.quantity)
        FROM gear_locker gl
        INNER JOIN gear_cost gc ON gc.class = gl.class
        INNER JOIN gear_items gi on gc.class = gi.class
        WHERE gl.member = $1 AND gi.global = false AND gc.priority = 0;",
        member.0.to_string(),
    );
    let res = query.fetch_one(executor).await?;
    #[allow(clippy::cast_possible_truncation)]
    Ok(res.sum.unwrap_or(0) as i32)
}
