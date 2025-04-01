use std::collections::HashMap;

use serenity::model::prelude::UserId;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

#[allow(clippy::ref_option)]
pub async fn get<'a, E>(
    member: &UserId,
    campaign: &Option<Uuid>,
    executor: E,
) -> Result<Option<String>, anyhow::Error>
where
    E: Executor<'a, Database = Postgres>,
{
    if let Some(campaign) = campaign {
        sqlx::query!(
            "SELECT loadout FROM campaigns_loadouts WHERE member = $1 AND campaign = $2 LIMIT 1",
            member.to_string(),
            campaign,
        )
        .fetch_optional(executor)
        .await?
        .map(|row| Ok(row.loadout))
        .transpose()
    } else {
        sqlx::query!(
            "SELECT loadout FROM gear_loadouts WHERE member = $1 LIMIT 1",
            member.to_string(),
        )
        .fetch_optional(executor)
        .await?
        .map(|row| Ok(row.loadout))
        .transpose()
    }
}

#[allow(clippy::ref_option)]
pub async fn store(
    member: &UserId,
    campaign: &Option<Uuid>,
    loadout: &str,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), anyhow::Error> {
    if let Some(campaign) = campaign {
        sqlx::query!(
            "INSERT INTO campaigns_loadouts (member, campaign, loadout) VALUES ($1, $2, $3) ON CONFLICT (member, campaign) DO UPDATE SET loadout = $3",
            member.to_string(),
            campaign,
            loadout,
        ).execute(&mut **executor).await?
    } else {
        sqlx::query!(
            "INSERT INTO gear_loadouts (member, loadout) VALUES ($1, $2) ON CONFLICT (member) DO UPDATE SET loadout = $2",
            member.to_string(),
            loadout,
        ).execute(&mut **executor).await?
    };
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
        "SELECT gc.class, gc.personal
        FROM gear_cost gc
        INNER JOIN gear_items gi on gi.class = gc.class
        WHERE gc.class=ANY($1);",
        &items
    );

    let res = query.fetch_all(executor).await?;

    let balance = res
        .iter()
        .map(|row| {
            #[allow(clippy::cast_possible_wrap)]
            loadout
                .get(&row.class)
                .map(|quantity| row.personal * *quantity as i32)
        })
        .sum::<Option<i32>>();

    Ok(balance.unwrap_or(0))
}
