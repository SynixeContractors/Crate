use std::net::SocketAddr;

use axum::{response::IntoResponse, routing::get, Router};
use chrono::{DateTime, Duration};
use icalendar::{Calendar, Component, Event, EventLike};
use synixe_events::missions::db::Response;
use synixe_proc::events_request_5;
use tokio::net::TcpListener;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();

    let app = Router::new().route("/calendar", get(calendar));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    debug!("Listening on {}", addr);
    axum::serve(
        TcpListener::bind(&addr).await.expect("bind to :3000"),
        app.into_make_service(),
    )
    .await
    .expect("should start server");
}

async fn calendar() -> impl IntoResponse {
    let Ok(Ok((Response::UpcomingSchedule(Ok(schedule)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await
    else {
        return "Error".to_string();
    };
    let mut calendar = Calendar::new();
    for scheduled in schedule {
        let start =
            DateTime::from_timestamp(scheduled.start.unix_timestamp(), 0).expect("valid timestamp");
        let briefing = scheduled.briefing();
        calendar.push(
            Event::new()
                .summary(&scheduled.name)
                .description(&briefing.get("old").cloned().unwrap_or_else(|| {
                    briefing
                        .get("mission")
                        .cloned()
                        .unwrap_or_else(|| "No briefing available.".to_string())
                }))
                .starts(start)
                .ends(start + chrono::Duration::try_minutes(150i64).expect("duration valid"))
                .done(),
        );
    }
    calendar
        .name("Synixe Contractors")
        .ttl(&Duration::try_hours(1).expect("duration valid"))
        .to_string()
}
