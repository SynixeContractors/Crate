#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use synixe_events::global::Publish;
use tokio_simple_scheduler::{Job, Scheduler};

#[macro_use]
extern crate tracing;

#[macro_use]
mod macros;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();

    let mut sched = Scheduler::default();

    bootstrap::NC::get().await;

    // Global
    event!(
        sched,
        "Global - Tick",
        "0 * * * * *",
        Publish::Tick {
            time: time::OffsetDateTime::now_utc()
        }
    );

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
        "0 0 * * * *",
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

    sched.start().await;

    info!("Done!");
}
