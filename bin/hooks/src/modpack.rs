use axum::http::StatusCode;
use synixe_proc::events_request_5;

pub async fn updated() -> StatusCode {
    let nats = bootstrap::NC::get().await;
    info!("modpack updated hook called");
    if let Err(e) = events_request_5!(nats, synixe_events::containers::modpack, Updated {}).await {
        error!("Failed to send Update event: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::OK
    }
}
