#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use synixe_proc::events_request;
use tokio_simple_scheduler::{Job, Scheduler};

#[macro_use]
extern crate log;

#[macro_use]
mod macros;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();
    bootstrap::tracer!("scheduler");

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
        "0 */20 * * * *",
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
        "0 */20 * * * *",
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

    // Certifications
    job!(
        sched,
        "Certifications - Check expired certifications",
        "0 0 12 * * *",
        synixe_events::certifications::executions,
        CheckExpiries
    );

    events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::executions,
        CheckRoles {}
    )
    .await
    .unwrap();

    sched.start().await;

    info!("Done!");
}
