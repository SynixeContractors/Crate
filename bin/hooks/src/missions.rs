use axum::http::StatusCode;
use synixe_proc::events_request;

pub async fn list_updated() -> StatusCode {
    let nats = bootstrap::NC::get().await;
    if let Err(e) = events_request!(nats, synixe_events::missions::db, UpdateMissionList {}).await {
        error!("Failed to send UpdateMissionList event: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::OK
    }
}
