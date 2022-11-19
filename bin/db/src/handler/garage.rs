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
            Self::FetchVehicleAssets { stored } => match stored {
                Some(stored) => {
                    fetch_as_and_respond!(
                        msg,
                        *db,
                        cx,
                        synixe_model::garage::VehicleAsset,
                        Response::FetchVehicleAssets,
                        "SELECT plate, stored FROM garage_vehicles WHERE stored = $1",
                        stored,
                    )?;
                    Ok(())
                }
                None => {
                    fetch_as_and_respond!(
                        msg,
                        *db,
                        cx,
                        synixe_model::garage::VehicleAsset,
                        Response::FetchVehicleAssets,
                        "SELECT plate, stored FROM garage_vehicles",
                    )?;
                    Ok(())
                }
            },
            Self::FetchVehicleAsset { plate } => {
                fetch_one_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::garage::VehicleAsset,
                    Response::FetchVehicleAsset,
                    "SELECT plate, stored FROM garage_vehicles WHERE plate = $1",
                    plate,
                )?;
                Ok(())
            }
            Self::FetchAllShopAssests {} => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::garage::ShopAsset,
                    Response::FetchAllShopAssests,
                    "SELECT name, cost, class FROM garage_shop",
                )?;
                Ok(())
            }
        }
    }
}
