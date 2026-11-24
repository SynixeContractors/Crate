use serenity::model::prelude::UserId;
use synixe_events::garage::db::{RepairGroupCost, VehicleDetails};
use uuid::Uuid;

// A vehicle's saved hits are `[name, damage]` pairs, see `objectState_save`. Each query reads them with
// `jsonb_array_elements`, as `[]` when the vehicle has none.

pub async fn vehicle_details(
    plate: Option<&str>,
    db: &sqlx::PgPool,
) -> Result<Vec<VehicleDetails>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"SELECT v.plate, s.name, s.class, v.stored, a.name AS "addon?", s.fuel_capacity, v.state, v.updated,
            COALESCE((
                SELECT SUM(CEIL(g.cost * (h->>1)::FLOAT8))
                FROM garage_repair_groups g
                CROSS JOIN LATERAL jsonb_array_elements(CASE WHEN jsonb_typeof(v.state->'hits') = 'array' THEN v.state->'hits' ELSE '[]'::JSONB END) h
                WHERE (h->>0) ILIKE ANY(g.hitpoints)
            ), 0)::INT AS "repair_cost!"
        FROM garage_vehicles v
        INNER JOIN garage_shop s ON s.id = v.id
        LEFT JOIN garage_shop a ON a.id = v.addon
        WHERE ($1::VARCHAR IS NULL OR v.plate = $1)
        ORDER BY s.name, v.plate"#,
        plate,
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| VehicleDetails {
            plate: row.plate,
            name: row.name,
            class: row.class,
            stored: row.stored,
            addon: row.addon,
            fuel_capacity: row.fuel_capacity,
            state: row.state,
            updated: row.updated,
            repair_cost: row.repair_cost,
        })
        .collect())
}

pub async fn repair_estimate(
    plate: &str,
    db: &sqlx::PgPool,
) -> Result<Vec<RepairGroupCost>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"SELECT g.id, g.name,
            COALESCE(jsonb_agg(jsonb_build_array(h->>0, (h->>1)::FLOAT8)) FILTER (WHERE h IS NOT NULL), '[]'::JSONB) AS "hitpoints!",
            COALESCE(SUM(CEIL(g.cost * (h->>1)::FLOAT8)), 0)::INT AS "cost!"
        FROM garage_repair_groups g
        LEFT JOIN garage_vehicles v ON v.plate = $1
        LEFT JOIN LATERAL jsonb_array_elements(CASE WHEN jsonb_typeof(v.state->'hits') = 'array' THEN v.state->'hits' ELSE '[]'::JSONB END) h
            ON (h->>0) ILIKE ANY(g.hitpoints)
        GROUP BY g.id
        ORDER BY g.name"#,
        plate,
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| RepairGroupCost {
            id: row.id,
            name: row.name,
            hitpoints: serde_json::from_value(row.hitpoints).unwrap_or_default(),
            cost: row.cost,
        })
        .collect())
}

/// Why a repair wasn't made, shown to the player
pub enum RepairError {
    NotFound,
    NotStored,
    Destroyed,
    NothingToRepair,
    Database(anyhow::Error),
}

impl From<sqlx::Error> for RepairError {
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e.into())
    }
}

impl std::fmt::Display for RepairError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NotFound => "Vehicle not found",
            Self::NotStored => "The vehicle has to be in the garage to be repaired",
            Self::Destroyed => "The vehicle is destroyed, there's nothing left to repair",
            Self::NothingToRepair => "Nothing in that part of the vehicle needs repairing",
            Self::Database(_) => "An unexpected error occurred, please try again later",
        })
    }
}

/// Repairs one group of a stored vehicle's parts, charging the company. Returns the group's name and the cost.
pub async fn repair(
    member: &UserId,
    plate: &str,
    group: Uuid,
    db: &sqlx::PgPool,
) -> Result<(String, i32), RepairError> {
    let mut tx = db.begin().await?;
    // Locked, so the vehicle can't be retrieved, or repaired twice, while this is charged
    let Some(vehicle) = sqlx::query!(
        "SELECT stored, state FROM garage_vehicles WHERE plate = $1 FOR UPDATE",
        plate,
    )
    .fetch_optional(&mut *tx)
    .await?
    else {
        return Err(RepairError::NotFound);
    };
    if !vehicle.stored {
        return Err(RepairError::NotStored);
    }
    if vehicle
        .state
        .as_ref()
        .and_then(|state| state.get("dead"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return Err(RepairError::Destroyed);
    }

    let Some(estimate) = sqlx::query!(
        r#"SELECT g.name, COUNT(h) AS "parts!", COALESCE(SUM(CEIL(g.cost * (h->>1)::FLOAT8)), 0)::INT AS "cost!"
        FROM garage_repair_groups g
        LEFT JOIN garage_vehicles v ON v.plate = $1
        LEFT JOIN LATERAL jsonb_array_elements(CASE WHEN jsonb_typeof(v.state->'hits') = 'array' THEN v.state->'hits' ELSE '[]'::JSONB END) h
            ON (h->>0) ILIKE ANY(g.hitpoints)
        WHERE g.id = $2
        GROUP BY g.id"#,
        plate,
        group,
    )
    .fetch_optional(&mut *tx)
    .await?
    else {
        return Err(RepairError::NothingToRepair);
    };
    if estimate.parts == 0 {
        return Err(RepairError::NothingToRepair);
    }

    // The repaired hitpoints are dropped from the saved state, so the vehicle spawns without that damage
    sqlx::query!(
        r#"UPDATE garage_vehicles v
        SET state = jsonb_set(v.state, '{hits}', COALESCE((
            SELECT jsonb_agg(h)
            FROM jsonb_array_elements(v.state->'hits') h
            WHERE NOT ((h->>0) ILIKE ANY(g.hitpoints))
        ), '[]'::JSONB)),
        updated = NOW()
        FROM garage_repair_groups g
        WHERE v.plate = $1 AND g.id = $2"#,
        plate,
        group,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO gear_bank_deposits (member, amount, reason, id) VALUES ('0', $1, $2, '00000000-0000-0000-0000-000000000003')",
        -estimate.cost,
        format!("Repair: {} on {plate} by {member}", estimate.name),
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO garage_log (plate, action, member, data) VALUES ($1, 'state', $2, $3)",
        plate,
        member.to_string(),
        serde_json::json!({ "repair": estimate.name, "cost": estimate.cost }),
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok((estimate.name, estimate.cost))
}
