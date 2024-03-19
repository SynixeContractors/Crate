use axum::{
    http,
    response::Html,
    routing::{get, get_service},
    Router,
};
use tera::Context;
use tower_http::services::ServeDir;

mod template;
mod vote;
use template::Template;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();

    let serve_dir = get_service(ServeDir::new("assets"));

    let app = Router::new()
        .route("/", get(dashboard))
        .nest("/vote", vote::router())
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
        .nest_service("/assets", serve_dir.clone())
        .fallback(|| async {
            Html(
                Template::get()
                    .render("error/404.html", &Context::new())
                    .unwrap_or_else(|e| {
                        error!("Error rendering template: {}", e);
                        "Error".to_string()
                    }),
            )
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
