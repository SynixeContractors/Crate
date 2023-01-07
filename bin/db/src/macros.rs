macro_rules! fetch_one_and_respond {
    ($msg:expr, $db:expr, $cx:expr, $respond:path, $query:expr, $($args:expr,)*) => {{
        let query = sqlx::query!(
            $query,
            $($args,)*
        );
        let res = query.fetch_one(&$db).await;
        match res {
            Ok(data) => {
                synixe_events::respond!($msg, $respond(Ok(Some(data.value)))).await?;
                Ok(())
            }
            Err(sqlx::Error::RowNotFound) => {
                synixe_events::respond!($msg, $respond(Ok(None))).await?;
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

macro_rules! fetch_all_and_respond {
    ($msg:expr, $db:expr, $cx:expr, $respond:path, $query:expr, $($args:expr,)*) => {{
        let query = sqlx::query!(
            $query,
            $($args,)*
        );
        let res = query.fetch_all(&$db).await;
        match res {
            Ok(data) => {
                synixe_events::respond!($msg, $respond(Ok(
                    data
                        .into_iter()
                        .map(|d| d.value)
                        .filter(|v| v.is_some())
                        .map(|v| v.unwrap())
                        .collect::<Vec<_>>()
                ))).await?;
                Ok(())
            }
            Err(sqlx::Error::RowNotFound) => {
                synixe_events::respond!($msg, $respond(Ok(Vec::new()))).await?;
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
        let query = sqlx::query_as!(
            $as,
            $query,
            $($args,)*
        );
        let res = query.fetch_all(&$db).await;
        match res {
            Ok(data) => {
                synixe_events::respond!($msg, $respond(Ok(data.clone()))).await?;
                Result::<_, anyhow::Error>::Ok(data)
            }
            Err(e) => {
                error!("{:?}", e);
                synixe_events::respond!($msg, $respond(Err(e.to_string()))).await?;
                Result::<_, anyhow::Error>::Err(e.into())
            }
        }
    }}
}

macro_rules! fetch_one_as_and_respond {
    ($msg:expr, $db:expr, $cx:expr, $as:path, $respond:path, $query:expr, $($args:expr,)*) => {{
        let query = sqlx::query_as!(
            $as,
            $query,
            $($args,)*
        );
        let res = query.fetch_one(&$db).await;
        match res {
            Ok(data) => {
                synixe_events::respond!($msg, $respond(Ok(Some(data.clone())))).await?;
                Result::<_, anyhow::Error>::Ok(data)
            }
            Err(e) => {
                error!("{:?}", e);
                synixe_events::respond!($msg, $respond(Err(e.to_string()))).await?;
                Result::<_, anyhow::Error>::Err(e.into())
            }
        }
    }}
}

macro_rules! execute_and_respond {
    ($msg:expr, $db:expr, $cx:expr, $respond:path, $query:expr, $($args:expr,)*) => {{
        let query = sqlx::query!(
            $query,
            $($args,)*
        );
        let res = query.execute(&$db).await;
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

macro_rules! match_no_return {
    ($query:expr, $typ:ident, $msg:expr, $cx:expr) => {{
        match $query.await {
            Ok(_) => {
                if let Err(e) = respond!($msg, Response::$typ(Ok(()))).await {
                    error!("Failed to respond to {}: {}", stringify!($typ), e);
                    return Err(e.into());
                }
                Result::<(), anyhow::Error>::Ok(())
            }
            Err(e) => {
                if let Err(e) = respond!($msg, Response::$typ(Err(e.to_string()))).await {
                    error!("Failed to respond to {}: {}", stringify!($typ), e);
                    return Err(e.into());
                }
                Err(e.into())
            }
        }
    }};
}

macro_rules! match_with_return {
    ($query:expr, $typ:ident, $msg:expr, $cx:expr) => {{
        match $query.await {
            Ok(data) => {
                if let Err(e) = respond!($msg, Response::$typ(Ok(data))).await {
                    error!("Failed to respond to {}: {}", stringify!($typ), e);
                    return Err(e.into());
                }
                Result::<(), anyhow::Error>::Ok(())
            }
            Err(e) => {
                if let Err(e) = respond!($msg, Response::$typ(Err(e.to_string()))).await {
                    error!("Failed to respond to {}: {}", stringify!($typ), e);
                    return Err(e.into());
                }
                Err(e.into())
            }
        }
    }};
}

macro_rules! transaction {
    ($db:expr, $msg:expr, $cx:expr) => {{
        match $db.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                if let Err(e) = respond!($msg, Response::LockerStore(Err(e.to_string()))).await {
                    error!("Failed to respond to LockerStore: {}", e);
                    return Err(e.into());
                }
                return Err(e.into());
            }
        }
    }};
}

macro_rules! quick_transaction {
    ($typ:ident, $db:expr, $msg:expr, $cx:expr, $func:expr, $($args:expr,)*) => {{
        let mut tx = transaction!($db, $msg, $cx);
        let res = match_no_return!($func($($args,)* &mut tx), $typ, $msg, $cx);
        if res.is_ok() {
            tx.commit().await?;
        }
        res
    }}
}

macro_rules! quick_transaction_return {
    ($typ:ident, $db:expr, $msg:expr, $cx:expr, $func:expr, $($args:expr,)*) => {{
        let mut tx = transaction!($db, $msg, $cx);
        let res = match_with_return!($func($($args,)* &mut tx), $typ, $msg, $cx);
        if res.is_ok() {
            tx.commit().await?;
        }
        res
    }}
}
