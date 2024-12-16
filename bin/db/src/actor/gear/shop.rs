use std::collections::HashMap;

use synixe_model::gear::Price;

pub async fn items(
    executor: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<HashMap<String, (Option<String>, Option<Vec<String>>, Price)>, anyhow::Error> {
    let query = sqlx::query!(
        "SELECT i.class, i.pretty, i.roles, cost_base.*, cost_current.* FROM gear_items i, LATERAL gear_item_base_cost(i.class) cost_base, LATERAL gear_item_current_cost(i.class) cost_current WHERE i.enabled = TRUE",
    );
    let res = query.fetch_all(&mut **executor).await?;
    Ok(res
        .into_iter()
        .filter(|row| row.personal.is_some())
        .map(|row| {
            (
                row.class,
                (
                    row.pretty,
                    row.roles
                        .map(|r| r.into_iter().filter(|r| !r.is_empty()).collect()),
                    Price::new(
                        row.personal.unwrap_or(-1),
                        row.company.unwrap_or(0),
                        row.personal_current,
                        row.company_current,
                        row.end_date,
                    ),
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
        "SELECT * FROM gear_item_base_cost($1), gear_item_current_cost($1)",
        item,
    );
    let res = query.fetch_one(&mut **executor).await?;
    Ok(Price::new(
        res.personal.unwrap_or(-1),
        res.company.unwrap_or(0),
        res.personal_current,
        res.company_current,
        res.end_date,
    ))
}
