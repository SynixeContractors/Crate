#[macro_export]
/// An event that expects a response.
macro_rules! request {
    ($nats:expr, $body:expr) => {{
        use $crate::Evokable;
        let body = $body;
        let path = body.self_path();
        let mut trace_body = $crate::Wrapper::new(body);
        $crate::opentelemetry::global::get_text_map_propagator(|injector| {
            injector.inject_context(&$crate::opentelemetry::Context::current(), &mut trace_body);
        });
        $nats.request_timeout(
            path,
            $crate::serde_json::to_vec(&trace_body).unwrap(),
            std::time::Duration::from_secs(2),
        )
    }};
}

#[macro_export]
/// An event that does not expect a response.
macro_rules! publish {
    ($nats:expr, $body:expr) => {{
        let body = $body;
        let path = body.self_path();
        let mut trace_body = $crate::Wrapper::new(None, body);
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
        $msg.respond($crate::serde_json::to_vec(&$resp).unwrap())
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
