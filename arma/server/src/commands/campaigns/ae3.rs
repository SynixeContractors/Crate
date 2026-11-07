use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex, PoisonError},
};

use arma_rs::{Context, Group, IntoArma};
use synixe_events::campaigns::db::Response;
use synixe_proc::events_request_5;
use uuid::Uuid;

use crate::{CONTEXT, RUNTIME};

/// Max characters per callback so each message stays well under the callExtension limit once escaped
const CHUNK_SIZE: usize = 4000;

/// Partial states sent from Arma, waiting for the final save call
static PARTS: LazyLock<Mutex<HashMap<Uuid, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn group() -> Group {
    Group::new()
        .command("part", part)
        .command("save", save)
        .command("restore", restore)
}

fn part(id: Uuid, chunk: String) {
    PARTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .entry(id)
        .or_default()
        .push_str(&chunk);
}

fn save(campaign: Uuid, id: Uuid, class: String) {
    let state = PARTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .remove(&id)
        .unwrap_or_default();
    if class.is_empty() || state.is_empty() {
        return;
    }
    RUNTIME.spawn(async move {
        let Ok(Ok((Response::StoreAe3Device(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            StoreAe3Device {
                campaign,
                id,
                class,
                state: serde_json::Value::String(state),
            }
        )
        .await
        else {
            error!("failed to save ae3 device");
            return;
        };
    });
}

/// Sends a stored state to Arma in chunks, followed by an `end` callback with the object class
pub fn send_state(context: &Context, id: Uuid, class: &str, state: &serde_json::Value) {
    let Some(text) = state.as_str() else {
        return;
    };
    let chars: Vec<char> = text.chars().collect();
    for chunk in chars.chunks(CHUNK_SIZE) {
        let chunk: String = chunk.iter().collect();
        if let Err(e) = context.callback_data(
            "crate:campaigns:ae3",
            "part",
            vec![id.to_arma(), chunk.to_arma()],
        ) {
            error!("error sending ae3:part: {e:?}");
            return;
        }
    }
    if let Err(e) = context.callback_data(
        "crate:campaigns:ae3",
        "end",
        vec![id.to_arma(), class.to_string().to_arma()],
    ) {
        error!("error sending ae3:end: {e:?}");
    }
}

fn restore(campaign: Uuid, id: Uuid) {
    RUNTIME.spawn(async move {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        let Ok(Ok((Response::Ae3DeviceById(Ok(devices)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            Ae3DeviceById { campaign, id }
        )
        .await
        else {
            error!("failed to load ae3 device {id}");
            return;
        };
        match devices.into_iter().next() {
            Some(device) => send_state(context, id, &device.class, &device.state),
            None => warn!("no ae3 device saved with id {id}"),
        }
        if let Err(e) = context.callback_data("crate:campaigns:ae3", "restore", vec![id.to_arma()])
        {
            error!("error sending ae3:restore: {e:?}");
        }
    });
}
