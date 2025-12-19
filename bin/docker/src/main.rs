use std::sync::LazyLock;

use synixe_meta::docker::ArmaServer;

#[macro_use]
extern crate tracing;

mod handler;
mod listener;

static CRATE_SERVER: LazyLock<(String, ArmaServer)> = LazyLock::new(|| {
    let server =
        std::env::var("CRATE_SERVER").expect("CRATE_SERVER must be set to the arma server name");
    (
        server.clone(),
        ArmaServer::try_from(server).expect("CRATE_SERVER must be a valid ArmaServer"),
    )
});
static CRATE_CONTAINER: LazyLock<String> = LazyLock::new(|| {
    std::env::var("CRATE_CONTAINER")
        .expect("CRATE_CONTAINER must be set to the arma server container name")
});

#[tokio::main]
async fn main() {
    bootstrap::logger::init();

    tokio::join!(listener::start(), handler::start());
}
