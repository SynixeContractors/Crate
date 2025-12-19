#[macro_use]
extern crate tracing;

#[macro_use]
mod macros;

mod actor;
mod handler;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();
    handler::start().await;
}

pub async fn game_audit(message: String) -> Option<serenity::all::MessageId> {
    match synixe_proc::events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::discord::write,
        GameAudit {
            message: synixe_events::discord::write::DiscordMessage {
                content: synixe_events::discord::write::DiscordContent::Text(message),
                reactions: Vec::new(),
            }
        }
    )
    .await
    {
        Ok(Ok((
            synixe_events::discord::write::Response::GameAudit(Ok((_channel, message))),
            _,
        ))) => {
            return Some(message);
        }
        Err(e) => {
            error!("Failed to audit: {}", e);
        }
        _ => {
            error!("Failed to audit: unknown error");
        }
    }
    None
}

pub async fn audit(message: String) -> Option<serenity::all::MessageId> {
    match synixe_proc::events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::discord::write,
        Audit {
            message: synixe_events::discord::write::DiscordMessage {
                content: synixe_events::discord::write::DiscordContent::Text(message),
                reactions: Vec::new(),
            }
        }
    )
    .await
    {
        Ok(Ok((synixe_events::discord::write::Response::Audit(Ok((_channel, message))), _))) => {
            return Some(message);
        }
        Err(e) => {
            error!("Failed to audit: {}", e);
        }
        _ => {
            error!("Failed to audit: unknown error");
        }
    }
    None
}
