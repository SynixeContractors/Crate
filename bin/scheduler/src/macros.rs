macro_rules! job {
    ($sched:expr, $name:expr, $cron:expr, $group:ty, $event:ident) => {
        $sched.add(
            Job::new($name, $cron, || {
                Box::pin(async {
                    use opentelemetry::trace::Tracer;
                    let tracer = bootstrap::tracer!("scheduler");
                    let _span =
                        tracer.start_with_context($name, &opentelemetry::Context::current());
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
