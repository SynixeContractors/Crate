use std::net::SocketAddr;

use axum::{extract::Path, response::IntoResponse, routing::get, Router, Server};
use synixe_events::gear::db::Response;
use synixe_proc::events_request_5;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();

    let app = Router::new().route("/api/bank/balance/:member", get(balance));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    debug!("Listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn balance(Path(id): Path<u64>) -> impl IntoResponse {
    let Ok(Ok((Response::BankBalance(Ok(Some(balance))), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankBalance {
            member: serenity::model::prelude::UserId(id),
        }
    )
    .await
    else {
        return "Error".to_string();
    };
    balance.to_string()
}
