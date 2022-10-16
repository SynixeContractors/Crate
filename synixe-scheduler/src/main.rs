#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use opentelemetry::sdk::propagation::TraceContextPropagator;
use rust_embed::RustEmbed;
use tokio_simple_scheduler::{Job, Scheduler};

#[macro_use]
extern crate log;

#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Assets;

mod jobs;

#[tokio::main]
async fn main() {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    bootstrap::logger::init();

    let mut sched = Scheduler::default();

    // Init NATS connection
    bootstrap::NC::get().await;

    sched.add(
        Job::new(
            "Recruiting - check steam forums for new posts",
            "0 */10 * * * *",
            || {
                Box::pin(async {
                    jobs::recruiting::check_steam_forums().await;
                })
            },
        )
        .unwrap(),
    );
    sched.add(
        Job::new(
            "Recruiting - check reddit findaunit for new posts",
            "0 */10 * * * *",
            || {
                Box::pin(async {
                    jobs::recruiting::check_reddit_findaunit().await;
                })
            },
        )
        .unwrap(),
    );
    sched.add(
        Job::new(
            "Recruiting - reddit findaunit post",
            "0 0 23 * * Thu,Fri,Sat",
            || {
                Box::pin(async {
                    jobs::recruiting::post_reddit_findaunit().await;
                })
            },
        )
        .unwrap(),
    );

    sched.start().await;

    info!("Done!");
}
