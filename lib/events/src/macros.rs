#[macro_export]
/// Handle the event.
macro_rules! handler {
    ($msg:expr, $nats:expr, $($events:ty),*) => {{
        use synixe_events::Evokable;
        use opentelemetry::trace::{Tracer, TraceContextExt};
        let subject = $msg.subject.clone();
        let sub = subject.as_str();
        $(
            if sub == <$events>::path() {
                let ((ev, _), pcx) = synixe_events::parse_data!($msg, $events);
                debug!("Handling event: {}", ev.name());
                let span = opentelemetry::global::tracer("handler").start_with_context(format!("{}:{}", sub.to_string(), ev.name()), &pcx);
                if let Err(e) = ev.handle($msg, $nats, pcx.with_span(span)).await {
                    error!("Error in handler {}: {}", sub, e);
                }
                continue
            }
        )*
    }}
}

#[macro_export]
/// Handle the event.
macro_rules! listener {
    ($msg:expr, $nats:expr, $($events:ty),*) => {{
        use synixe_events::Publishable;
        use opentelemetry::trace::{Tracer, TraceContextExt};
        let subject = $msg.subject.clone();
        let sub = subject.as_str();
        $(
            if sub == <$events>::path() {
                let ((ev, _), pcx) = synixe_events::parse_data!($msg, $events);
                debug!("Handling event: {}", ev.name());
                let span = opentelemetry::global::tracer("handler").start_with_context(format!("{}:{}", sub.to_string(), ev.name()), &pcx);
                if let Err(e) = ev.listen($msg, $nats, pcx.with_span(span)).await {
                    error!("Error in handler {}: {}", sub, e);
                }
                continue
            }
        )*
    }}
}

#[macro_export]
/// An event that does not expect a response.
macro_rules! publish {
    ($nats:expr, $body:expr) => {{
        use $crate::Publishable;
        let body = $body;
        let path = body.self_path();
        trace!("publishing on {:?}", path);
        let mut trace_body = $crate::Wrapper::new(body);
        $crate::opentelemetry::global::get_text_map_propagator(|injector| {
            injector.inject_context(&$crate::opentelemetry::Context::current(), &mut trace_body);
        });
        $nats.publish(path, $crate::serde_json::to_vec(&trace_body).unwrap())
    }};
}

#[macro_export]
/// Respond to a request.
macro_rules! respond {
    ($msg:expr, $resp:expr) => {{
        use $crate::opentelemetry::trace::Tracer;
        let _span = $crate::opentelemetry::global::tracer("respond")
            .start_with_context("respond", &$crate::opentelemetry::Context::current());
        let mut trace_body = $crate::Wrapper::new($resp);
        $crate::opentelemetry::global::get_text_map_propagator(|injector| {
            injector.inject_context(&$crate::opentelemetry::Context::current(), &mut trace_body);
        });
        $msg.respond($crate::serde_json::to_vec(&trace_body).unwrap())
    }};
}

#[macro_export]
/// Unwraps an event.
macro_rules! parse_data {
    ($msg:expr, $t:ty) => {{
        let wrapper = $crate::serde_json::from_slice::<$crate::Wrapper<$t>>(&$msg.data).unwrap();
        let parent_context = $crate::opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&wrapper)
        });
        (wrapper.into_parts(), parent_context)
    }};
}
