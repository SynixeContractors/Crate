macro_rules! job {
    ($sched:expr, $name:expr, $cron:expr, $group:ty, $event:ident) => {
        $sched.add(
            Job::new($name, $cron, || {
                Box::pin(async {
                    use opentelemetry::trace::{TraceContextExt, Tracer};
                    let tracer = bootstrap::tracer!("scheduler");
                    let span = tracer.start_with_context($name, &opentelemetry::Context::current());
                    let cx = opentelemetry::Context::current().with_span(span);
                    cx.attach();
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
