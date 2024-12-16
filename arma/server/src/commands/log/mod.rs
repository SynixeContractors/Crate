use arma_rs::Group;
use synixe_proc::events_request_5;

use crate::{RUNTIME, SERVER_ID};

pub fn group() -> Group {
    Group::new()
        .command("connected", connected)
        .command("disconnected", disconnected)
        .command("chat", chat)
        .command("take", take)
        .command("role", role)
}

fn connected(steam: String, name: String) {
    RUNTIME.spawn(async move {
        if let Err(e) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::servers::db,
            Log {
                server: (*SERVER_ID).clone(),
                steam,
                action: "connected".to_string(),
                data: serde_json::json!({
                    "name": name,
                }),
            }
        )
        .await
        {
            error!("failed to log server event: {}", e);
        }
    });
}

fn disconnected(steam: String, name: String) {
    RUNTIME.spawn(async move {
        if let Err(e) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::servers::db,
            Log {
                server: (*SERVER_ID).clone(),
                steam,
                action: "disconnected".to_string(),
                data: serde_json::json!({
                    "name": name,
                }),
            }
        )
        .await
        {
            error!("failed to log server event: {}", e);
        }
    });
}

fn chat(steam: String, name: String, channel: String, message: String) {
    RUNTIME.spawn(async move {
        if let Err(e) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::servers::db,
            Log {
                server: (*SERVER_ID).clone(),
                steam,
                action: "chat".to_string(),
                data: serde_json::json!({
                    "channel": channel,
                    "name": name,
                    "message": message,
                }),
            }
        )
        .await
        {
            error!("failed to log server event: {}", e);
        }
    });
}

fn take(steam: String, name: String, from: String, item: String) {
    RUNTIME.spawn(async move {
        if let Err(e) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::servers::db,
            Log {
                server: (*SERVER_ID).clone(),
                steam,
                action: "take".to_string(),
                data: serde_json::json!({
                    "name": name,
                    "item": item,
                    "from": from,
                }),
            }
        )
        .await
        {
            error!("failed to log server event: {}", e);
        }
    });
}

fn role(steam: String, name: String, discord: String, role: String) {
    RUNTIME.spawn(async move {
        if let Err(e) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::servers::db,
            Log {
                server: (*SERVER_ID).clone(),
                steam,
                action: "role".to_string(),
                data: serde_json::json!({
                    "name": name,
                    "discord": discord,
                    "role": role,
                }),
            }
        )
        .await
        {
            error!("failed to log server event: {}", e);
        }
    });
}
