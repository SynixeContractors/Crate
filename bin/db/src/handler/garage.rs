use async_trait::async_trait;
use synixe_events::{
    garage::db::{FetchedPlate, Request, Response},
    gear::db::FuelType,
    respond,
};
use synixe_proc::events_request_5;

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::FetchPlates { search } => {
                let search = search.clone().unwrap_or_default();
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    FetchedPlate,
                    Response::FetchPlates,
                    "SELECT
                        plate
                    FROM
                        garage_vehicles
                    WHERE
                        LOWER(plate) LIKE LOWER($1)",
                    format!("{search}"),
                )?;
                Ok(())
            }
            Self::FetchStoredVehicles { stored, plate } => {
                let plate = plate.clone().unwrap_or_default();
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::garage::VehicleAsset,
                    Response::FetchStoredVehicles,
                    r#"
                    SELECT
                        v.plate,
                        v.id,
                        v.addon,
                        v.stored,
                        s.name,
                        s.class,
                        (SELECT COUNT(base) FROM garage_shop WHERE base = s.id) as addons
                    FROM
                        garage_vehicles v
                    INNER JOIN
                        garage_shop s
                    ON
                        s.id = v.id
                    WHERE
                        LOWER(plate) LIKE LOWER($1)
                        AND ($2 OR stored = $3)
                    "#,
                    format!("%{plate}%"),
                    stored.is_none(),
                    stored.unwrap_or_default(),
                )?;
                Ok(())
            }
            Self::FetchStoredVehicle { plate } => {
                fetch_one_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::garage::VehicleAsset,
                    Response::FetchStoredVehicle,
                    r#"
                    SELECT
                        v.plate,
                        v.id,
                        v.addon,
                        v.stored,
                        s.name,
                        s.class,
                        (SELECT COUNT(base) FROM garage_shop WHERE base = s.id) as addons
                    FROM
                        garage_vehicles v
                    INNER JOIN
                        garage_shop s
                    ON
                        s.id = v.id
                    WHERE plate = $1"#,
                    plate,
                )?;
                Ok(())
            }
            Self::FetchStoredAddons { plate } => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::garage::ShopAsset,
                    Response::FetchStoredAddons,
                    r#"
                    SELECT
                        a.id,
                        s.name,
                        s.cost,
                        s.class,
                        s.base,
                        s.plate_template,
                        s.fuel_capacity
                    FROM
                        garage_addons a
                    INNER JOIN
                        garage_shop s
                    ON
                        s.id = a.id
                    WHERE
                        s.base = (SELECT v.id FROM garage_vehicles v WHERE v.plate = $1)
                        AND a.count > 0"#,
                    plate,
                )?;
                Ok(())
            }
            Self::FetchShopAssets { search } => {
                let search = search.clone().unwrap_or_default();
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::garage::ShopAsset,
                    Response::FetchShopAssets,
                    "SELECT
                        *
                    FROM
                        garage_shop
                    WHERE
                        LOWER(name) LIKE LOWER($1)",
                    format!("%{search}%"),
                )?;
                Ok(())
            }
            Self::FetchShopAsset { asset } => {
                fetch_one_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::garage::ShopAsset,
                    Response::FetchShopAsset,
                    "SELECT
                        *
                    FROM
                        garage_shop
                    WHERE
                        LOWER(name) LIKE LOWER($1)",
                    format!("%{asset}%"),
                )?;
                Ok(())
            }
            Self::FetchVehicleColors { id } => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::garage::VehicleColor,
                    Response::FetchVehicleColors,
                    "SELECT
                        *
                    FROM
                        garage_colors
                    WHERE
                        id = $1",
                    id,
                )?;
                Ok(())
            }
            Self::PurchaseShopAsset { order } => match order {
                synixe_events::garage::db::ShopOrder::Vehicle { id, color, member } => {
                    let query = sqlx::query!(
                        "INSERT INTO garage_purchases (id, color, member, plate) VALUES ($1, $2, $3, generate_plate($1)) RETURNING plate",
                        id,
                        color.as_ref(),
                        member.to_string(),
                    );
                    let plate = query.fetch_one(&*db).await?.plate;
                    respond!(msg, Response::PurchaseShopAsset(Ok(plate.clone()))).await?;
                    #[allow(clippy::cast_sign_loss)]
                    let amount =
                        sqlx::query!("SELECT fuel_capacity FROM garage_shop WHERE id = $1", id,)
                            .fetch_one(&*db)
                            .await?
                            .fuel_capacity as u64;
                    if amount == 0 {
                        return Ok(());
                    }
                    events_request_5!(
                        bootstrap::NC::get().await,
                        synixe_events::gear::db,
                        Fuel {
                            member: *member,
                            amount,
                            fuel_type: FuelType::Regular,
                            plate,
                            map: String::new()
                        }
                    )
                    .await
                    .map_err(|e| anyhow::anyhow!(e))??;
                    Ok(())
                }
                synixe_events::garage::db::ShopOrder::Addon { id, member } => {
                    let query = sqlx::query!(
                        "INSERT INTO garage_purchases (id, member) VALUES ($1, $2)",
                        id,
                        member.to_string(),
                    );
                    query.execute(&*db).await?;
                    respond!(msg, Response::PurchaseShopAsset(Ok(None))).await?;
                    Ok(())
                }
            },
            Self::FetchVehicleInfo { plate } => {
                fetch_one_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_events::garage::db::SpawnInfo,
                    Response::FetchVehicleInfo,
                    "SELECT
                        (
                            SELECT
                                class
                            FROM
                                garage_shop s
                            WHERE
                                s.id = COALESCE(v.addon, v.id)
                        ) as class,
                        state
                    FROM
                        garage_vehicles v
                    WHERE
                        v.plate = $1",
                    plate,
                )?;
                Ok(())
            }
            Self::AttachAddon {
                plate,
                addon,
                member,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::AttachAddon,
                    "INSERT into garage_log (plate, action, member, data) VALUES ($1, 'attach', $2, $3)",
                    plate,
                    member.to_string(),
                    serde_json::json!({ "addon": addon }),
                )
            }
            Self::DetachAddon { plate, member } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::DetachAddon,
                    "INSERT into garage_log (plate, action, member) VALUES ($1, 'detach', $2)",
                    plate,
                    member.to_string(),
                )
            }

            Self::RetrieveVehicle { plate, member } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::RetrieveVehicle,
                    "INSERT into garage_log (plate, action, member) VALUES ($1, 'retrieve', $2)",
                    plate,
                    member.to_string(),
                )
            }
            Self::StoreVehicle {
                plate,
                state,
                member,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::StoreVehicle,
                    "INSERT into garage_log (plate, action, member, data) VALUES ($1, 'store', $2, $3)",
                    plate,
                    member.to_string(),
                    state,
                )
            }
        }
    }
}
