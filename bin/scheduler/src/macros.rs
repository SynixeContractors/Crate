macro_rules! job {
    ($sched:expr, $name:expr, $cron:expr, $group:ty, $event:ident) => {
        $sched.add(
            Job::new($name, $cron, || {
                Box::pin(async {
                    info!("job `{}`", $name);
                    if let Err(e) =
                        events_request!(bootstrap::NC::get().await, $group, $event {}).await
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
