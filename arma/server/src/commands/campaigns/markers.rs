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

fn save(campaign: Uuid, name: String, data: HashMap<String, Value>) {
    RUNTIME.spawn(async move {
        let Ok(Ok((Response::StoreMarker(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            StoreMarker {
                campaign,
                name,
                data: serde_json::Value::Object(
                    data.into_iter().map(|(k, v)| (k, v.to_json())).collect()
                ),
            }
        )
        .await
        else {
            error!("failed to save marker");
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
        let Ok(Ok((Response::Markers(Ok(markers)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            Markers { campaign }
        )
        .await
        else {
            error!("failed to load markers");
            return;
        };
        debug!("loading {} markers", markers.len());
        for marker in markers {
            if let Err(e) = context.callback_data(
                "crate:campaigns:markers",
                "load",
                vec![marker.name.to_arma(), marker.data.to_arma()],
            ) {
                error!("error sending campaigns:markers:load: {:?}", e);
            }
        }
        if let Err(e) = context.callback_null("crate:campaigns:markers", "done") {
            error!("error sending campaigns:markers:done: {:?}", e);
        }
    });
}

fn delete(campaign: Uuid, name: String) {
    RUNTIME.spawn(async move {
        let Ok(Ok((Response::DeleteMarker(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            DeleteMarker { campaign, name }
        )
        .await
        else {
            error!("failed to delete marker");
            return;
        };
    });
}
