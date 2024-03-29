use axum::http::StatusCode;
use synixe_proc::events_request_5;

#[allow(clippy::cognitive_complexity)]
pub async fn list_updated() -> StatusCode {
    let nats = bootstrap::NC::get().await;
    info!("missions list updated hook called");
    if let Err(e) = events_request_5!(nats, synixe_events::missions::db, UpdateMissionList {}).await
    {
        error!("Failed to send UpdateMissionList event to db: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    } else if let Err(e) = events_request_5!(
        nats,
        synixe_events::containers::missions,
        UpdateMissionList {}
    )
    .await
    {
        error!("Failed to send UpdateMissionList event to container: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::OK
    }
}
