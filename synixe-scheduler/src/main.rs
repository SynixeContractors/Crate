#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use opentelemetry::sdk::propagation::TraceContextPropagator;
use tokio_simple_scheduler::{Job, Scheduler};

#[macro_use]
extern crate log;

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

    jobs::recruiting::check_steam_forums().await;
    jobs::recruiting::check_reddit_findaunit().await;

    sched.start().await;

    info!("Done!");
}
