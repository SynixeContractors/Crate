#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

mod discord;

#[tokio::main]
async fn main() {
    let bot = discord::build().await;
    discord::start(bot).await;
}
