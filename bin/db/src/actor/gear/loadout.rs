use std::collections::HashMap;

use serenity::model::prelude::UserId;
use sqlx::{Executor, Postgres};

pub async fn get<'a, E>(member: &UserId, executor: E) -> Result<Option<String>, anyhow::Error>
where
    E: Executor<'a, Database = Postgres>,
{
    let query = sqlx::query!(
        "SELECT loadout FROM gear_loadouts WHERE member = $1 LIMIT 1",
        member.0.to_string(),
    );
    query
        .fetch_optional(executor)
        .await?
        .map(|row| Ok(row.loadout))
        .transpose()
}

pub async fn store(
    member: &UserId,
    loadout: &str,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        "INSERT INTO gear_loadouts (member, loadout) VALUES ($1, $2) ON CONFLICT (member) DO UPDATE SET loadout = $2",
        member.0.to_string(),
        loadout,
    );
    query.execute(&mut *executor).await?;
    Ok(())
}

pub async fn balance<'a, E>(
    loadout: HashMap<String, u32>,
    executor: E,
) -> Result<i32, anyhow::Error>
where
    E: Executor<'a, Database = Postgres>,
{
    let items = loadout
        .keys()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    let query = sqlx::query!(
        "SELECT gc.class, gc.cost
        FROM gear_cost gc
        INNER JOIN gear_items gi on gi.class = gc.class
        WHERE gc.class=ANY($1) AND gi.global = false;",
        &items
    );

    let res = query.fetch_all(executor).await?;

    let balance = res
        .iter()
        .map(|row| {
            #[allow(clippy::cast_possible_wrap)]
            loadout
                .get(&row.class)
                .map(|quantity| row.cost * *quantity as i32)
        })
        .sum::<Option<i32>>();

    Ok(balance.unwrap_or(0))
}
