use axum::{extract::Path, response::Html, routing::get, Router};
use tera::Context;

use crate::template::Template;

pub fn router() -> Router {
    Router::new()
        .route("/", get(missions))
        .route("/:mission", get(mission))
}

async fn missions() -> Html<String> {
    Html(
        Template::get()
            .render("missions/index.html", &Context::new())
            .unwrap_or_else(|e| {
                error!("Error rendering template: {}", e);
                "Error".to_string()
            }),
    )
}

async fn mission(Path(id): Path<String>) -> Html<String> {
    Html(
        Template::get()
            .render("missions/member.html", &Context::new())
            .unwrap_or_else(|e| {
                error!("Error rendering template: {}", e);
                "Error".to_string()
            }),
    )
}
