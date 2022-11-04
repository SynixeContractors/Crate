macro_rules! trace_query {
    ($cx:expr, $query:expr, $($args:expr,)*) => ({
        use opentelemetry::trace::{Span, Tracer};
        let mut span = opentelemetry::global::tracer("trace_query").start_with_context($query, &$cx);
        span.set_attribute(opentelemetry::KeyValue::new("query", $query));
        let mut i = 0;
        $(
            i += 1;
            span.set_attribute(opentelemetry::KeyValue::new(format!("arg-{}", i), format!("{:?}", $args)));
        )*
        (sqlx::sqlx_macros::expand_query!(source = $query, args = [$($args,)*]), span)
    })
}

macro_rules! trace_query_as {
    ($cx:expr, $out_struct:path, $query:expr, $($args:expr,)*) => ( {
        use opentelemetry::trace::{Span, Tracer};
        let mut span = opentelemetry::global::tracer("trace_query_as").start_with_context($query, &$cx);
        span.set_attribute(opentelemetry::KeyValue::new("query", $query));
        #[allow(unused_mut, unused_variables)]
        let mut i = 0;
        $(
            i += 1;
            span.set_attribute(opentelemetry::KeyValue::new(format!("arg-{}", i), format!("{:?}", $args)));
        )*
        (sqlx::sqlx_macros::expand_query!(record = $out_struct, source = $query, args = [$($args,)*]), span)
    })
}

macro_rules! fetch_one_and_respond {
    ($msg:expr, $db:expr, $cx:expr, $respond:path, $query:expr, $($args:expr,)*) => {{
        use opentelemetry::trace::Span;
        let (query, mut span) = trace_query!(
            $cx,
            $query,
            $($args,)*
        );
        let res = query.fetch_one(&$db).await;
        span.end();
        match res {
            Ok(data) => {
                synixe_events::respond!($msg, $respond(Ok(data.value))).await?;
                Ok(())
            }
            Err(e) => {
                error!("{:?}", e);
                synixe_events::respond!($msg, $respond(Err(e.to_string()))).await?;
                Err(e.into())
            }
        }
    }}
}

macro_rules! fetch_as_and_respond {
    ($msg:expr, $db:expr, $cx:expr, $as:path, $respond:path, $query:expr, $($args:expr,)*) => {{
        use opentelemetry::trace::Span;
        let (query, mut span) = trace_query_as!(
            $cx,
            $as,
            $query,
            $($args,)*
        );
        let res = query.fetch_all(&$db).await;
        span.end();
        match res {
            Ok(data) => {
                synixe_events::respond!($msg, $respond(Ok(data))).await?;
                Ok(())
            }
            Err(e) => {
                error!("{:?}", e);
                synixe_events::respond!($msg, $respond(Err(e.to_string()))).await?;
                Err(e.into())
            }
        }
    }}
}

macro_rules! fetch_one_as_and_respond {
    ($msg:expr, $db:expr, $cx:expr, $as:path, $respond:path, $query:expr, $($args:expr,)*) => {{
        use opentelemetry::trace::Span;
        let (query, mut span) = trace_query_as!(
            $cx,
            $as,
            $query,
            $($args,)*
        );
        let res = query.fetch_one(&$db).await;
        span.end();
        match res {
            Ok(data) => {
                synixe_events::respond!($msg, $respond(Ok(Some(data)))).await?;
                Ok(())
            }
            Err(e) => {
                error!("{:?}", e);
                synixe_events::respond!($msg, $respond(Err(e.to_string()))).await?;
                Err(e.into())
            }
        }
    }}
}

macro_rules! execute_and_respond {
    ($msg:expr, $db:expr, $cx:expr, $respond:path, $query:expr, $($args:expr,)*) => {{
        use opentelemetry::trace::Span;
        let (query, mut span) = trace_query!(
            $cx,
            $query,
            $($args,)*
        );
        let res = query.execute(&$db).await;
        span.end();
        match res {
            Ok(_) => {
                synixe_events::respond!($msg, $respond(Ok(()))).await?;
                Ok(())
            }
            Err(e) => {
                error!("{:?}", e);
                synixe_events::respond!($msg, $respond(Err(e.to_string()))).await?;
                Err(e.into())
            }
        }
    }}
}
