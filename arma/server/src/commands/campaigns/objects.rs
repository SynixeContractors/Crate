use std::collections::HashMap;

use arma_rs::{Group, IntoArma, Value};
use synixe_events::campaigns::db::Response;
use synixe_proc::events_request_5;
use uuid::Uuid;

use crate::{CONTEXT, RUNTIME};

pub fn group() -> Group {
    Group::new().command("save", save).command("load", load)
}

fn save(campaign: Uuid, id: Uuid, class: String, data: HashMap<String, Value>) {
    RUNTIME.spawn(async move {
        let Ok(Ok((Response::StoreObject(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            StoreObject {
                campaign,
                id,
                class,
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
        let Ok(Ok((Response::Objects(Ok(objects)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            Objects {
                campaign,
            }
        ).await else {
            error!("failed to load objects");
            return;
        };
        for object in objects {
            context.callback_data(
                "crate:campaigns:objects",
                "load",
                vec![
                    object.id.to_arma(),
                    object.class.to_arma(),
                    object.data.to_arma(),
                ],
            );
        }
        context.callback_null("crate:campaigns:objects", "done");
    });
}
