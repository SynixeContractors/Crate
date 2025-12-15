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
            error!("failed to log server event: {e}");
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
            error!("failed to log server event: {e}");
        }
    });
}

fn chat(steam: String, name: String, channel: String, message: String) {
    if message.is_empty() {
        return;
    }
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
            error!("failed to log server event: {e}");
        }
        audit(steam, "Chat".to_string(), format!("said \"{message}\"")).await;
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
            error!("failed to log server event: {e}");
        }
        audit(
            steam,
            "Item Taken".to_string(),
            format!("took {item} from {from}"),
        )
        .await;
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
            error!("failed to log server event: {e}");
        }
    });
}

async fn audit(steam: String, header: String, message: String) {
    let Ok(Ok((synixe_events::discord::db::Response::FromSteam(Ok(Some(discord))), _))) =
        events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::discord::db,
            FromSteam {
                steam: steam.clone(),
            }
        )
        .await
    else {
        error!("failed to get discord from steam");
        return;
    };
    if let Err(e) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::discord::write,
        GameAudit {
            message: DiscordMessage {
                content: DiscordContent::Text(format!("**{header}**\n<@{discord}> {message}")),
                reactions: Vec::new(),
            }
        }
    )
    .await
    {
        error!("failed to log server event: {e}");
    }
}
