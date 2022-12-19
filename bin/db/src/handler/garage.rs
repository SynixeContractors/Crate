use async_trait::async_trait;
use synixe_events::garage::db::{Request, Response};

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
                        s.class
                    FROM 
                        garage_vehicles v 
                    INNER JOIN 
                        garage_shop s 
                    ON
                        s.id = v.id 
                    WHERE 
                        plate LIKE $1
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
                        s.class
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
                        s.base
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
                        name LIKE $1",
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
                        name Like $1",
                    format!("%{asset}%"),
                )?;
                Ok(())
            }
            Self::PurchaseShopAsset { plate, id, member } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::PurchaseShopAsset,
                    "INSERT INTO garage_purchases (id, plate, member) VALUES ($1, $2, $3)",
                    id,
                    plate.as_ref(),
                    member.to_string(),
                )
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
        }
    }
}
