use synixe_events::global::Publish;
use tokio_simple_scheduler::{Job, Scheduler};

macro_rules! job {
    ($sched:expr, $name:expr, $cron:expr, $group:ty, $event:ident) => {
        $sched.add(
            Job::new($name, $cron, || {
                Box::pin(async {
                    info!("job `{}`", $name);
                    if let Err(e) = synixe_proc::events_request_5!(
                        bootstrap::NC::get().await,
                        $group,
                        $event {}
                    )
                    .await
                    {
                        error!("error during `{}`: {:?}", $name, e);
                    }
                })
            })
            .unwrap(),
        );
    };
}

macro_rules! event {
    ($sched:expr, $name:expr, $cron:expr, $event:expr) => {
        $sched.add(
            Job::new($name, $cron, || {
                Box::pin(async {
                    info!("event `{}`", $name);
                    if let Err(e) =
                        synixe_events::publish!(bootstrap::NC::get().await, $event).await
                    {
                        error!("error during `{}`: {:?}", $name, e);
                    }
                })
            })
            .unwrap(),
        );
    };
}

pub fn create() -> Scheduler {
    let mut sched = Scheduler::default();

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
        "0 0 19 * * Thu,Fri,Sat",
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
        "0 0 19 * * *",
        synixe_events::certifications::executions,
        CheckExpiries
    );

    // Activity
    job!(
        sched,
        "Activity - Check for new activity",
        "0 0 19 * * *",
        synixe_events::discord::executions,
        UpdateActivityRoles
    );

    // Tips
    job!(
        sched,
        "Tips - post weekly tip",
        "0 0 19 * * Sat,Sun",
        synixe_events::discord::executions,
        PostWeeklyTips
    );

    sched
}
