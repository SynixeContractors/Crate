#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use opentelemetry::sdk::propagation::TraceContextPropagator;
use synixe_proc::events_request;
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

    // Recruiting
    sched.add(
        Job::new(
            "Recruiting - check steam forums for new posts",
            "0 */10 * * * *",
            || {
                Box::pin(async {
                    if let Err(e) = events_request!(
                        bootstrap::NC::get().await,
                        synixe_events::recruiting::executions,
                        CheckSteam {}
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
                    if let Err(e) = events_request!(
                        bootstrap::NC::get().await,
                        synixe_events::recruiting::executions,
                        CheckReddit {}
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
                    if let Err(e) = events_request!(
                        bootstrap::NC::get().await,
                        synixe_events::recruiting::executions,
                        PostReddit {}
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

    // Missions
    sched.add(
        Job::new("Missions - Update mission list", "0 */30 * * * *", || {
            Box::pin(async {
                if let Err(e) = events_request!(
                    bootstrap::NC::get().await,
                    synixe_events::missions::db,
                    UpdateMissionList {}
                )
                .await
                {
                    error!("error updating mission list: {:?}", e);
                }
            })
        })
        .unwrap(),
    );

    sched.start().await;

    info!("Done!");
}
