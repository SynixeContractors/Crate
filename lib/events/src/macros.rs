#[macro_export]
/// Handle the event.
macro_rules! handler {
    ($msg:expr, $nats:expr, $($events:ty),*,) => {{
        use synixe_events::Evokable;
        let subject = $msg.subject.clone();
        let sub = subject.as_str();
        info!("seen event: {}", sub);
        if 1 == 2 {}
        $(
            else if sub == <$events>::path() {
                let Ok((ev, _)) = synixe_events::parse_data!($msg, $events) else {
                    error!("Failed to parse event: {}", sub);
                    continue
                };
                info!("Handling event: {:?}", ev);
                if let Err(e) = ev.handle($msg, $nats).await {
                    error!("Error in handler {}: {}", sub, e);
                }
            }
        )*
    }}
}

#[macro_export]
/// Handle the event.
macro_rules! listener {
    ($msg:expr, $nats:expr, $($events:ty),*,) => {{
        use synixe_events::Publishable;
        let subject = $msg.subject.clone();
        let sub = subject.as_str();
        $(
            if sub == <$events>::path() {
                let Ok((ev, pcx)) = synixe_events::parse_data!($msg, $events) else {
                    error!("Failed to parse event: {}", sub);
                    continue
                };
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
        #[allow(deprecated)]
        $nats.publish(
            path,
            $crate::serde_json::to_vec(&trace_body).unwrap().into(),
        )
    }};
}

#[macro_export]
/// Respond to a request.
macro_rules! respond {
    ($msg:expr, $resp:expr) => {{
        Box::pin(async {
            let mut trace_body = $crate::Wrapper::new($resp);
            let nats = bootstrap::NC::get().await;
            let response = nats
                .request(
                    $msg.reply.clone().unwrap(),
                    $crate::serde_json::to_vec(&trace_body).unwrap().into(),
                )
                .await;
            response.map(|_| ())
        })
    }};
}

#[macro_export]
/// Unwraps an event.
macro_rules! parse_data {
    ($msg:expr, $t:ty) => {{ $crate::serde_json::from_slice::<$crate::Wrapper<$t>>(&$msg.payload).map(|d| d.into_parts()) }};
}
