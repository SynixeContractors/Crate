use std::net::SocketAddr;

use axum::{
    extract::Path,
    http::{self, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, get_service},
    Router, Server,
};
use synixe_events::gear::db::Response;
use synixe_proc::events_request_5;
use template::Template;
use tera::Context;
use tower_http::services::ServeDir;

#[macro_use]
extern crate tracing;

mod members;
mod template;

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

#[tokio::main]
async fn main() {
    bootstrap::logger::init();

    let serve_dir = get_service(ServeDir::new("assets")).handle_error(handle_error);

    let app = Router::new()
        .route("/", get(dashboard))
        .nest("/members", members::router())
        .route(
            "/tailwind.css",
            get(|| async {
                let mut response: http::Response<String> =
                    http::Response::new(include_str!("../tailwind.css").into());
                *response.status_mut() = http::StatusCode::OK;
                response.headers_mut().insert(
                    http::header::CONTENT_TYPE,
                    http::header::HeaderValue::from_static("text/css"),
                );
                response.headers_mut().insert(
                    http::header::CACHE_CONTROL,
                    http::header::HeaderValue::from_static("max-age=31536000"),
                );
                response.headers_mut().insert(
                    http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                    http::header::HeaderValue::from_static("*"),
                );
                response
            }),
        )
        .nest_service("/assets", serve_dir.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    debug!("Listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn dashboard() -> Html<String> {
    Html(
        Template::get()
            .render("dashboard.html", &Context::new())
            .unwrap_or_else(|e| {
                error!("Error rendering template: {}", e);
                "Error".to_string()
            }),
    )
}

async fn balance(Path(id): Path<u64>) -> Html<String> {
    let Ok(Ok((Response::BankBalance(Ok(Some(balance))), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankBalance {
            member: serenity::model::prelude::UserId(id),
        }
    )
    .await else {
        return Html("Error".to_string());
    };
    let mut context = Context::new();
    context.insert("balance", &balance);
    Html(
        Template::get()
            .render("pages/profile.html", &context)
            .unwrap_or_else(|e| {
                error!("Error rendering template: {}", e);
                "Error".to_string()
            }),
    )
}
