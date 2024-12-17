use arma_rs::Group;
use synixe_events::discord::write::{DiscordContent, DiscordMessage};
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
                steam: steam.clone(),
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
        if let Err(e) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::discord::write,
            GameAudit {
                message: DiscordMessage {
                    content: DiscordContent::Text(format!(
                        "**Chat Message**\n<@{steam}> said \"{message}\""
                    )),
                    reactions: Vec::new(),
                }
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
                steam: steam.clone(),
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
        if let Err(e) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::discord::write,
            GameAudit {
                message: DiscordMessage {
                    content: DiscordContent::Text(format!(
                        "**Item Taken**\n<@{steam}> took {item} from {from}"
                    )),
                    reactions: Vec::new(),
                }
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
