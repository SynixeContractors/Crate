use yew::prelude::*;

use actix_web::{get, App as ActixApp, Error, HttpResponse, HttpServer};
use tokio::task::LocalSet;
use tokio::task::spawn_blocking;


#[function_component]
fn App() -> Html {
    html! {<div>{"Hello, World!"}</div>}
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = HttpServer::new(|| ActixApp::new().service(render));
    server.bind(("127.0.0.1", 8080))?.run().await
}

#[get("/")]
async fn render() -> Result<HttpResponse, Error> {
    let content = spawn_blocking(move || {
        use tokio::runtime::Builder;
        let set = LocalSet::new();

        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        set.block_on(&rt, async {
            let renderer = yew::ServerRenderer::<App>::new();

            renderer.render().await
        })
    })
    .await
    .expect("the thread has failed.");

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            r#"<!DOCTYPE HTML>
                <html>
                    <head>
                        <title>yew-ssr with actix-web example</title>
                    </head>
                    <body>
                        <h1 style="background-color:powderblue;">yew-ssr with actix-web example</h1>
                        {}
                    </body>
                </html>
            "#,
            content
        )))
}