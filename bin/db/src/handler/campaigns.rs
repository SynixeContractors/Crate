use async_trait::async_trait;
use synixe_events::campaigns::db::{Request, Response};

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: async_nats::message::Message,
        _nats: async_nats::Client,
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
                            campaigns_objects
                        WHERE
                            campaign = $1"#,
                    campaign,
                )?;
                Ok(())
            }
            Self::Ae3DevicesByIds { ids } => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::campaigns::Ae3Device,
                    Response::Ae3DevicesByIds,
                    r#"
                        SELECT
                            id,
                            class,
                            state
                        FROM
                            campaigns_ae3_devices
                        WHERE
                            id = ANY($1)"#,
                    ids,
                )?;
                Ok(())
            }
            Self::Ae3DeviceById { id } => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::campaigns::Ae3Device,
                    Response::Ae3DeviceById,
                    r#"
                        SELECT
                            id,
                            class,
                            state
                        FROM
                            campaigns_ae3_devices
                        WHERE
                            id = $1"#,
                    id,
                )?;
                Ok(())
            }
            Self::Ae3PackedItems {} => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::campaigns::Ae3PackedItem,
                    Response::Ae3PackedItems,
                    r#"
                        SELECT
                            p.item_class,
                            p.device_id,
                            d.class
                        FROM
                            campaigns_ae3_packed_items p
                        INNER JOIN
                            campaigns_ae3_devices d ON d.id = p.device_id"#,
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
                            campaigns_groups
                        WHERE
                            campaign = $1"#,
                    campaign,
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
                            campaigns_units
                        WHERE
                            campaign = $1"#,
                    campaign,
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
                            campaigns_markers
                        WHERE
                            campaign = $1"#,
                    campaign,
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
                    "INSERT INTO
                        campaigns_objects (id, campaign, class, data)
                    VALUES
                        ($1, $2, $3, $4)
                    ON CONFLICT (id, campaign) DO UPDATE SET
                        class = $3,
                        data = $4",
                    id,
                    campaign,
                    class,
                    data,
                )
            }
            Self::StoreAe3Device { id, class, state } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::StoreAe3Device,
                    "INSERT INTO
                        campaigns_ae3_devices (id, class, state)
                    VALUES
                        ($1, $2, $3)
                    ON CONFLICT (id) DO UPDATE SET
                        class = $2,
                        state = $3,
                        updated_at = NOW()",
                    id,
                    class,
                    state,
                )
            }
            Self::StoreAe3PackedItem {
                item_class,
                device_id,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::StoreAe3PackedItem,
                    "INSERT INTO
                        campaigns_ae3_packed_items (item_class, device_id)
                    VALUES
                        ($1, $2)
                    ON CONFLICT (item_class) DO UPDATE SET
                        device_id = $2,
                        updated_at = NOW()",
                    item_class,
                    device_id,
                )
            }
            Self::DeleteAe3PackedItem { item_class } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::DeleteAe3PackedItem,
                    "DELETE FROM campaigns_ae3_packed_items WHERE item_class = $1",
                    item_class,
                )
            }
            Self::StoreGroup { campaign, id, data } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::StoreGroup,
                    "INSERT INTO
                        campaigns_groups (id, campaign, data)
                    VALUES
                        ($1, $2, $3)
                    ON CONFLICT (id, campaign) DO UPDATE SET
                        data = $3",
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
                    r#"INSERT INTO
                        campaigns_units (id, campaign, class, "group", data)
                    VALUES
                        ($1, $2, $3, $4, $5)
                    ON CONFLICT (id, campaign) DO UPDATE SET
                        class = $3,
                        "group" = $4,
                        data = $5"#,
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
                    "INSERT INTO
                        campaigns_markers (name, campaign, data)
                    VALUES
                        ($1, $2, $3)
                        ON CONFLICT (name, campaign) DO UPDATE SET
                        data = $3",
                    name,
                    campaign,
                    data,
                )
            }
            Self::DeleteObject { campaign, id } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::DeleteObject,
                    "DELETE FROM campaigns_objects WHERE id = $1 AND campaign = $2",
                    id,
                    campaign,
                )
            }
            Self::DeleteGroup { campaign, id } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::DeleteGroup,
                    "DELETE FROM campaigns_groups WHERE id = $1 AND campaign = $2",
                    id,
                    campaign,
                )
            }
            Self::DeleteUnit { campaign, id } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::DeleteUnit,
                    "DELETE FROM campaigns_units WHERE id = $1 AND campaign = $2",
                    id,
                    campaign,
                )
            }
            Self::DeleteMarker { campaign, name } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::DeleteMarker,
                    "DELETE FROM campaigns_markers WHERE name = $1 AND campaign = $2",
                    name,
                    campaign,
                )
            }
        }
    }
}
