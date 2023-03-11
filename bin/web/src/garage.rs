use axum::{extract::Path, response::Html, routing::get, Router};
use tera::Context;

use crate::template::Template;

pub fn router() -> Router {
    Router::new()
        .route("/", get(garage))
        .route("/:plate", get(mission))
}

async fn garage() -> Html<String> {
    Html(
        Template::get()
            .render("garage/index.html", &Context::new())
            .unwrap_or_else(|e| {
                error!("Error rendering template: {}", e);
                "Error".to_string()
            }),
    )
}

async fn mission(Path(id): Path<String>) -> Html<String> {
    Html(
        Template::get()
            .render("garage/asset.html", &Context::new())
            .unwrap_or_else(|e| {
                error!("Error rendering template: {}", e);
                "Error".to_string()
            }),
    )
}
