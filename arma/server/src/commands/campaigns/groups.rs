use std::collections::HashMap;

use arma_rs::{Group, IntoArma, Value};
use synixe_events::campaigns::db::Response;
use synixe_proc::events_request_5;
use uuid::Uuid;

use crate::{CONTEXT, RUNTIME};

pub fn group() -> Group {
    Group::new().command("save", save).command("load", load)
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
        for group in groups {
            context.callback_data(
                "crate:campaigns:groups",
                "load",
                vec![group.id.to_arma(), group.data.to_arma()],
            );
        }
        context.callback_null("crate:campaigns:groups", "done");
    });
}
