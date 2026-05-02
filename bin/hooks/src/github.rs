use axum::http::StatusCode;
use synixe_events::publish;

pub async fn handler(data: String) -> Result<(), StatusCode> {
    let nats = bootstrap::NC::get().await;
    if let Err(e) = publish!(nats, synixe_events::github::publish::Publish::Hook { data }).await {
        error!("Failed to publish GitHub hook event: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    info!("Published GitHub hook event");

    Ok(())
}
