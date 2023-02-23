use axum::{extract::Path, response::Html, routing::get, Router};
use tera::Context;

use crate::template::Template;

pub fn router() -> Router {
    Router::new()
        .route("/", get(members))
        .route("/:member", get(member))
}

async fn members() -> Html<String> {
    Html(
        Template::get()
            .render("members/index.html", &Context::new())
            .unwrap_or_else(|e| {
                error!("Error rendering template: {}", e);
                "Error".to_string()
            }),
    )
}

async fn member(Path(id): Path<u64>) -> Html<String> {
    Html(
        Template::get()
            .render("members/member.html", &Context::new())
            .unwrap_or_else(|e| {
                error!("Error rendering template: {}", e);
                "Error".to_string()
            }),
    )
}
