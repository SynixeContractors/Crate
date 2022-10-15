#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use tokio_simple_scheduler::{Job, Scheduler};

mod jobs;

#[tokio::main]
async fn main() {
    let mut sched = Scheduler::default();

    // Init NATS connection
    // NC::get().await;

    sched.add(
        Job::new(
            "Recruiting - check steam forums for new posts",
            "1/10 * * * * *",
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
            "1/10 * * * * *",
            || {
                Box::pin(async {
                    jobs::recruiting::check_reddit_findaunit().await;
                })
            },
        )
        .unwrap(),
    );

    sched.start().await;

    println!("Done!");
}
