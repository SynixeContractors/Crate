use std::net::SocketAddr;

use axum::{extract::Path, response::IntoResponse, routing::get, Router};
use synixe_events::gear::db::Response;
use synixe_proc::events_request_5;
use tokio::net::TcpListener;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();

    let app = Router::new().route("/api/bank/balance/:member", get(balance));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    debug!("Listening on {}", addr);
    axum::serve(
        TcpListener::bind(&addr).await.expect("bind to addr :3000"),
        app.into_make_service(),
    )
    .await
    .expect("failed to serve");
}

async fn balance(Path(id): Path<u64>) -> impl IntoResponse {
    let Ok(Ok((Response::BankBalance(Ok(Some(balance))), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankBalance {
            member: serenity::model::prelude::UserId::new(id),
        }
    )
    .await
    else {
        return "Error".to_string();
    };
    balance.to_string()
}
