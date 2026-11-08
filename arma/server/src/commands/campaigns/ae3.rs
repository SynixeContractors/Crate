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
        .command("itemDeployed", item_deployed)
        .command("itemsLoad", items_load)
}

fn part(id: Uuid, chunk: String) {
    PARTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .entry(id)
        .or_default()
        .push_str(&chunk);
}

/// Stores the state sent by `part`, and records the inventory item the device is packed as, if any
fn save(id: Uuid, class: String, item_class: String) {
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

        // The packed item references its device row, so it is only recorded after the device is stored
        if item_class.is_empty() {
            return;
        }
        let Ok(Ok((Response::StoreAe3PackedItem(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            StoreAe3PackedItem {
                item_class,
                device_id: id,
            }
        )
        .await
        else {
            error!("failed to save ae3 packed item");
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

fn restore(id: Uuid) {
    RUNTIME.spawn(async move {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        let Ok(Ok((Response::Ae3DeviceById(Ok(devices)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            Ae3DeviceById { id }
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

/// Records that a packed item was redeployed, so the mapping is no longer needed
fn item_deployed(item_class: String) {
    RUNTIME.spawn(async move {
        let Ok(Ok((Response::DeleteAe3PackedItem(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            DeleteAe3PackedItem { item_class }
        )
        .await
        else {
            error!("failed to delete ae3 packed item");
            return;
        };
    });
}

/// Sends every packed device to Arma so AE3's in-memory item buffers (lost on a restart) can be
/// rebuilt, letting a packed item still be redeployed
fn items_load() {
    RUNTIME.spawn(async move {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        let Ok(Ok((Response::Ae3PackedItems(Ok(items)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            Ae3PackedItems {}
        )
        .await
        else {
            error!("failed to load ae3 packed items");
            return;
        };
        for item in items {
            // Sent ahead of the item so a flash drive's filesystem is in place before AE3 mounts it on connect
            match events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::campaigns::db,
                Ae3DeviceById { id: item.device_id }
            )
            .await
            {
                Ok(Ok((Response::Ae3DeviceById(Ok(devices)), _))) => {
                    if let Some(device) = devices.into_iter().next() {
                        send_state(context, item.device_id, &device.class, &device.state);
                    }
                }
                _ => error!("failed to load ae3 device {}", item.device_id),
            }
            if let Err(e) = context.callback_data(
                "crate:campaigns:ae3",
                "itemLoad",
                vec![
                    item.item_class.to_arma(),
                    item.device_id.to_arma(),
                    item.class.to_arma(),
                ],
            ) {
                error!("error sending ae3:itemLoad: {e:?}");
            }
        }
    });
}
