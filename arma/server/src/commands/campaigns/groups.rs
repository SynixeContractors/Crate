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

fn save(campaign: Uuid, id: Uuid, data: HashMap<String, Value>) {
    RUNTIME.spawn(async move {
        let Ok(Ok((Response::StoreGroup(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            StoreGroup {
                campaign,
                id,
                data: serde_json::Value::Object(
                    data
                        .into_iter()
                        .map(|(k, v)| (k, v.to_json()))
                        .collect()
                ),
            }
        ).await else {
            error!("failed to save group");
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
        let Ok(Ok((Response::Groups(Ok(groups)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            Groups {
                campaign,
            }
        ).await else {
            error!("failed to load groups");
            return;
        };
        debug!("loading {} groups", groups.len());
        for group in groups {
            if let Err(e) = context.callback_data(
                "crate:campaigns:groups",
                "load",
                vec![group.id.to_arma(), group.data.to_arma()],
            ) {
                error!("error sending groups:load: {:?}", e);
            }
        }
        if let Err(e) = context.callback_null("crate:campaigns:groups", "done") {
            error!("error sending groups:done: {:?}", e);
        }
    });
}

fn delete(campaign: Uuid, id: Uuid) {
    RUNTIME.spawn(async move {
        let Ok(Ok((Response::DeleteGroup(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            DeleteGroup {
                campaign,
                id,
            }
        ).await else {
            error!("failed to delete group");
            return;
        };
    });
}
