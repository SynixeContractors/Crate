#[macro_export]
/// Handle the event.
macro_rules! handler {
    ($msg:expr, $nats:expr, $($events:ty),*) => {{
        use synixe_events::Evokable;
        let subject = $msg.subject.clone();
        let sub = subject.as_str();
        $(
            if sub == <$events>::path() {
                let (ev, _) = synixe_events::parse_data!($msg, $events);
                debug!("Handling event: {}", ev.name());
                if let Err(e) = ev.handle($msg, $nats).await {
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
        let subject = $msg.subject.clone();
        let sub = subject.as_str();
        $(
            if sub == <$events>::path() {
                let (ev, pcx) = synixe_events::parse_data!($msg, $events);
                debug!("Handling event: {}", ev.name());
                if let Err(e) = ev.listen($msg, $nats).await {
                    error!("Error in handler {}: {}", sub, e);
                }
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
        $nats.publish(path, $crate::serde_json::to_vec(&trace_body).unwrap())
    }};
}

#[macro_export]
/// Respond to a request.
macro_rules! respond {
    ($msg:expr, $resp:expr) => {{
        let mut trace_body = $crate::Wrapper::new($resp);
        $msg.respond($crate::serde_json::to_vec(&trace_body).unwrap())
    }};
}

#[macro_export]
/// Unwraps an event.
macro_rules! parse_data {
    ($msg:expr, $t:ty) => {{
        $crate::serde_json::from_slice::<$crate::Wrapper<$t>>(&$msg.data)
            .unwrap()
            .into_parts()
    }};
}
