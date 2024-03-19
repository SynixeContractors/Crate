#[macro_use]
extern crate tracing;

mod active;
mod closed;
mod create;
mod db;
mod discord;
mod input;
mod pending;

#[tokio::main]
async fn main() {
    main_menu().await;
}

async fn main_menu() {
    loop {
        match input::select(
            &[
                "Create Poll",
                "Pending Polls",
                "Active Polls",
                "Closed Polls",
                "Exit",
            ],
            "Main Menu",
        ) {
            "Create Poll" => create::menu().await,
            "Pending Polls" => pending::menu().await,
            "Active Polls" => active::menu().await,
            "Closed Polls" => closed::menu().await,
            "Exit" => std::process::exit(0),
            _ => unreachable!(),
        }
    }
}
