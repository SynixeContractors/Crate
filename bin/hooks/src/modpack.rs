use axum::http::StatusCode;
use synixe_proc::events_request;

pub async fn updated() -> StatusCode {
    let nats = bootstrap::NC::get().await;
    if let Err(e) = events_request!(nats, synixe_events::containers::modpack, Updated {}).await {
        error!("Failed to send Update event: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::OK
    }
}
