#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use opentelemetry::sdk::propagation::TraceContextPropagator;
use synixe_events::{recruiting, request};
use tokio_simple_scheduler::{Job, Scheduler};

#[macro_use]
extern crate log;

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
                    if let Err(e) = request!(
                        bootstrap::NC::get().await,
                        recruiting::executions::Request::CheckSteam {}
                    )
                    .await
                    {
                        error!("error checking on steam: {:?}", e);
                    }
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
                    if let Err(e) = request!(
                        bootstrap::NC::get().await,
                        recruiting::executions::Request::CheckReddit {}
                    )
                    .await
                    {
                        error!("error checking on reddit: {:?}", e);
                    }
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
                    if let Err(e) = request!(
                        bootstrap::NC::get().await,
                        recruiting::executions::Request::PostReddit {}
                    )
                    .await
                    {
                        error!("error posting to reddit: {:?}", e);
                    }
                })
            },
        )
        .unwrap(),
    );

    sched.start().await;

    info!("Done!");
}
