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

fn save(campaign: Uuid, id: Uuid, class: String, data: HashMap<String, Value>) {
    if class.is_empty() {
        return;
    }
    RUNTIME.spawn(async move {
        let Ok(Ok((Response::StoreObject(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            StoreObject {
                campaign,
                id,
                class,
                data: serde_json::Value::Object(
                    data.into_iter().map(|(k, v)| (k, v.to_json())).collect()
                ),
            }
        )
        .await
        else {
            error!("failed to save object");
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
            Objects { campaign }
        )
        .await
        else {
            error!("failed to load objects");
            return;
        };
        debug!("loading {} objects", objects.len());
        for object in objects {
            if let Err(e) = context.callback_data(
                "crate:campaigns:objects",
                "load",
                vec![
                    object.id.to_arma(),
                    object.class.to_arma(),
                    object.data.to_arma(),
                ],
            ) {
                error!("error sending objects:load: {:?}", e);
            }
        }
        if let Err(e) = context.callback_null("crate:campaigns:objects", "done") {
            error!("error sending objects:done: {:?}", e);
        }
    });
}

fn delete(campaign: Uuid, id: Uuid) {
    RUNTIME.spawn(async move {
        let Ok(Ok((Response::DeleteObject(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            DeleteObject { campaign, id }
        )
        .await
        else {
            error!("failed to delete object");
            return;
        };
    });
}
