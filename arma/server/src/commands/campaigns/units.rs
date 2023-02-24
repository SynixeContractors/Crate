use std::collections::HashMap;

use arma_rs::{Group, IntoArma, Value};
use synixe_events::campaigns::db::Response;
use synixe_proc::events_request_5;
use uuid::Uuid;

use crate::{CONTEXT, RUNTIME};

pub fn group() -> Group {
    Group::new()
        .command("save", save)
        .command("load", load)
        .command("delete", delete)
}

fn save(campaign: Uuid, id: Uuid, class: String, group: Uuid, data: HashMap<String, Value>) {
    RUNTIME.spawn(async move {
        let Ok(Ok((Response::StoreUnit(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            StoreUnit {
                campaign,
                id,
                class,
                group,
                data: serde_json::Value::Object(
                    data
                        .into_iter()
                        .map(|(k, v)| (k, v.to_json()))
                        .collect()
                ),
            }
        ).await else {
            error!("failed to save unit");
            return;
        };
    });
}

fn load(campaign: Uuid) {
    RUNTIME.spawn(async move {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        let Ok(Ok((Response::Units(Ok(units)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            Units {
                campaign,
            }
        ).await else {
            error!("failed to load units");
            return;
        };
        debug!("loading {} units", units.len());
        for unit in units {
            context.callback_data(
                "crate:campaigns:units",
                "load",
                vec![
                    unit.id.to_arma(),
                    unit.class.to_arma(),
                    unit.group.to_arma(),
                    unit.data.to_arma(),
                ],
            );
        }
        context.callback_null("crate:campaigns:units", "done");
    });
}

fn delete(campaign: Uuid, id: Uuid) {
    RUNTIME.spawn(async move {
        let Ok(Ok((Response::DeleteUnit(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            DeleteUnit {
                campaign,
                id,
            }
        ).await else {
            error!("failed to delete unit");
            return;
        };
    });
}
