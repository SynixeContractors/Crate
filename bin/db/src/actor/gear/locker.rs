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
