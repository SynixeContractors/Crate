macro_rules! job {
    ($sched:expr, $name:expr, $cron:expr, $group:ty, $event:ident) => {
        $sched.add(
            Job::new($name, $cron, || {
                Box::pin(async {
                    use opentelemetry::trace::{FutureExt, Span, TraceContextExt, Tracer};
                    let tracer = bootstrap::tracer!("scheduler");
                    let mut span = tracer.start($name);
                    span.set_attribute(opentelemetry::KeyValue::new(
                        "cron".to_string(),
                        $cron.to_string(),
                    ));
                    let cx = opentelemetry::Context::new().with_span(span);
                    if let Err(e) = events_request!(bootstrap::NC::get().await, $group, $event {})
                        .with_context(cx)
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
