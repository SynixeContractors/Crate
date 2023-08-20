use std::collections::HashMap;

use synixe_model::gear::Price;

pub async fn items(
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<HashMap<String, (Option<String>, Option<Vec<String>>, Price)>, anyhow::Error> {
    let query = sqlx::query!(
        "SELECT i.class, i.pretty, i.roles, i.global, gear_item_base_cost(i.class) as base, c.cost, c.end_date FROM gear_items i, LATERAL gear_item_current_cost(i.class) c WHERE i.enabled = TRUE",
    );
    let res = query.fetch_all(&mut **executor).await?;
    Ok(res
        .into_iter()
        .filter(|row| row.base.is_some())
        .map(|row| {
            (
                row.class,
                (
                    row.pretty,
                    row.roles
                        .map(|r| r.into_iter().filter(|r| !r.is_empty()).collect()),
                    Price::new(row.base.unwrap(), row.cost, row.end_date, row.global),
                ),
            )
        })
        .collect())
}

pub async fn price(
    item: &str,
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Price, anyhow::Error> {
    let query = sqlx::query!(
        "SELECT i.global, gear_item_base_cost(i.class) as base, c.cost, c.end_date FROM gear_items i, LATERAL gear_item_current_cost(i.class) c WHERE i.class = $1",
        item,
    );
    let res = query.fetch_one(&mut **executor).await?;
    Ok(Price::new(
        res.base.unwrap_or(-1),
        res.cost,
        res.end_date,
        res.global,
    ))
}
