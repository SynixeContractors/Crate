#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

#[macro_use]
extern crate tracing;

mod active;
mod closed;
mod create;
mod pending;

use dialoguer::Select;

#[tokio::main]
async fn main() {
    main_menu().await;
}

async fn main_menu() {
    loop {
        let option = Select::new().with_prompt("Main Menu").items(&[
            "Create Poll",
            "Pending Polls",
            "Active Polls",
            "Closed Polls",
            "Exit",
        ]);
        match option.interact().unwrap() {
            0 => create::menu().await,
            1 => pending::menu().await,
            2 => active::menu().await,
            3 => closed::menu().await,
            4 => std::process::exit(0),
            _ => unreachable!(),
        }
    }
}
