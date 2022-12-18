use std::net::SocketAddr;

use axum::{response::IntoResponse, routing::get, Router, Server};
use chrono::NaiveDateTime;
use icalendar::{Calendar, Component, Event, EventLike};
use synixe_events::missions::db::Response;
use synixe_proc::events_request;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();

    let app = Router::new().route("/calendar", get(calendar));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    debug!("Listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn calendar() -> impl IntoResponse {
    let Ok(Ok((Response::UpcomingSchedule(Ok(schedule)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await else {
        return "Error".to_string();
    };
    let mut calendar = Calendar::new();
    for scheduled in schedule {
        let Ok(Ok((Response::FetchMission(Ok(Some(mission))), _))) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            FetchMission { mission: scheduled.mission }
        ).await else {
            return "Error".to_string();
        };
        let start = NaiveDateTime::from_timestamp_opt(scheduled.start.unix_timestamp(), 0).unwrap();
        calendar.push(
            Event::new()
                .summary(&mission.name)
                .description(&mission.description)
                .starts(start)
                .ends(start + chrono::Duration::minutes(150i64))
                .done(),
        );
    }
    calendar.to_string()
}
