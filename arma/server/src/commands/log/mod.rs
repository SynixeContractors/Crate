use arma_rs::Group;
use synixe_proc::events_request_5;

use crate::RUNTIME;

pub fn group() -> Group {
    Group::new()
        .command("connected", connected)
        .command("disconnected", disconnected)
        .command("chat", chat)
        .command("role", role)
}

fn connected(steam: String, name: String) {
    RUNTIME.spawn(async move {
        if let Err(e) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::servers::db,
            Log {
                server: "arma".to_string(),
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
                server: "arma".to_string(),
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

fn chat(steam: String, name: String, message: String) {
    RUNTIME.spawn(async move {
        if let Err(e) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::servers::db,
            Log {
                server: "arma".to_string(),
                steam,
                action: "chat".to_string(),
                data: serde_json::json!({
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

fn role(steam: String, name: String, discord: String, role: String) {
    RUNTIME.spawn(async move {
        if let Err(e) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::servers::db,
            Log {
                server: "arma".to_string(),
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
