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
