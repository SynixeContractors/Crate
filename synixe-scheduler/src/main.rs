#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use opentelemetry::sdk::propagation::TraceContextPropagator;
use synixe_proc::events_request;
use tokio_simple_scheduler::{Job, Scheduler};

#[macro_use]
extern crate log;

#[macro_use]
mod macros;

#[tokio::main]
async fn main() {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    bootstrap::logger::init();

    let mut sched = Scheduler::default();

    // Init NATS connection
    bootstrap::NC::get().await;

    // Recruiting
    job!(
        sched,
        "Recruiting - check steam for new posts",
        "0 */10 * * * *",
        synixe_events::recruiting::executions,
        CheckSteam
    );
    job!(
        sched,
        "Recruiting - check reddit findaunit for new posts",
        "0 */10 * * * *",
        synixe_events::recruiting::executions,
        CheckReddit
    );
    job!(
        sched,
        "Recruiting - reddit findaunit post",
        "0 0 23 * * Thu,Fri,Sat",
        synixe_events::recruiting::executions,
        PostReddit
    );

    // Missions
    job!(
        sched,
        "Missions - Update mission list",
        "0 */30 * * * *",
        synixe_events::missions::db,
        UpdateMissionList
    );
    job!(
        sched,
        "Missions - Post about upcoming missons",
        "0 */5 * * * *",
        synixe_events::missions::executions,
        PostUpcomingMissions
    );

    sched.start().await;

    info!("Done!");
}
