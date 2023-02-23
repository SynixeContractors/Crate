use async_trait::async_trait;
use synixe_events::campaigns::db::{Request, Response};

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
            Self::Objects { campaign } => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::campaigns::Object,
                    Response::Objects,
                    r#"
                        SELECT
                            id,
                            campaign,
                            class,
                            data
                        FROM
                            campaigns_objects"#,
                )?;
                Ok(())
            }
            Self::Groups { campaign } => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::campaigns::Group,
                    Response::Groups,
                    r#"
                        SELECT
                            id,
                            campaign,
                            data
                        FROM
                            campaigns_groups"#,
                )?;
                Ok(())
            }
            Self::Units { campaign } => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::campaigns::Unit,
                    Response::Units,
                    r#"
                        SELECT
                            id,
                            campaign,
                            class,
                            "group",
                            data
                        FROM
                            campaigns_units"#,
                )?;
                Ok(())
            }
            Self::Markers { campaign } => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::campaigns::Marker,
                    Response::Markers,
                    r#"
                        SELECT
                            name,
                            campaign,
                            data
                        FROM
                            campaigns_markers"#,
                )?;
                Ok(())
            }
            Self::StoreObject {
                campaign,
                id,
                class,
                data,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::StoreObject,
                    "INSERT INTO campaigns_objects (id, campaign, class, data) VALUES ($1, $2, $3, $4)",
                    id,
                    campaign,
                    class,
                    data,
                )
            }
            Self::StoreGroup { campaign, id, data } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::StoreGroup,
                    "INSERT INTO campaigns_groups (id, campaign, data) VALUES ($1, $2, $3)",
                    id,
                    campaign,
                    data,
                )
            }
            Self::StoreUnit {
                campaign,
                id,
                class,
                group,
                data,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::StoreUnit,
                    r#"INSERT INTO campaigns_units (id, campaign, class, "group", data) VALUES ($1, $2, $3, $4, $5)"#,
                    id,
                    campaign,
                    class,
                    group,
                    data,
                )
            }
            Self::StoreMarker {
                campaign,
                name,
                data,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::StoreMarker,
                    "INSERT INTO campaigns_markers (name, campaign, data) VALUES ($1, $2, $3)",
                    name,
                    campaign,
                    data,
                )
            }
        }
    }
}
