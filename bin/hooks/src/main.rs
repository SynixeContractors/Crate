#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use std::net::SocketAddr;

use axum::{
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::post,
    Router, Server,
};

#[macro_use]
extern crate tracing;

mod missions;
mod modpack;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();

    let app = Router::new().nest(
        "/hooks",
        Router::new()
            .route("/missions/list_updated", post(missions::list_updated))
            .route("/modpack/updated", post(modpack::updated))
            .route_layer(middleware::from_fn(check_token)),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    debug!("Listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn check_token<B: Send>(req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let token = std::env::var("HOOKS_TOKEN").expect("HOOKS_TOKEN must be set");
    let Some(token_header) = req.headers().get("X-Token") else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    if token_header.to_str().unwrap() == token {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
