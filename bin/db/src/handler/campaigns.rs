use async_trait::async_trait;
use synixe_events::campaigns::db::Request;

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::Objects { campaign } => todo!(),
            Self::Groups { campaign } => todo!(),
            Self::Units { campaign } => todo!(),
            Self::Markers { campaign } => todo!(),
            Self::StoreObject {
                campaign,
                id,
                class,
                data,
            } => todo!(),
            Self::StoreGroup { campaign, id, data } => todo!(),
            Self::StoreUnit { campaign, id, data } => todo!(),
            Self::StoreMarker {
                campaign,
                name,
                data,
            } => todo!(),
        }
    }
}
