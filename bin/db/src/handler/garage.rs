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
            Self::FetchVehicleAssets { stored, plate } => {
                let plate = plate.clone().unwrap_or_default();
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::garage::VehicleAsset,
                    Response::FetchVehicleAssets,
                    r#"
                    SELECT 
                        v.plate, 
                        v.stored, 
                        v.id,
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
            Self::FetchVehicleAsset { plate } => {
                fetch_one_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::garage::VehicleAsset,
                    Response::FetchVehicleAsset,
                    r#"
                    SELECT 
                        v.plate, 
                        v.stored, 
                        v.id,
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
            Self::FetchAllShopAssests { search } => {
                let search = search.clone().unwrap_or_default();
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::garage::ShopAsset,
                    Response::FetchAllShopAssests,
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
            Self::PurchaseVehicleAsset { plate, id, member } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::PurchaseVehicleAsset,
                    "INSERT INTO garage_purchases (plate, id, member, cost) VALUES ($1, $2, $3)",
                    plate,
                    id,
                    member.to_string(),
                )
            }
        }
    }
}
