use async_trait::async_trait;
use synixe_events::{
    recruiting::{
        db,
        executions::{Request, Response},
    },
    respond,
};
use synixe_proc::events_request;

use super::Handler;

mod candidate;
mod reddit;
mod steam;

const IGNORE: [&str; 5] = ["exile", "vietnam", "police rp", "halo", "ww2"];
const PING: [&str; 2] = ["pmc", "persistent"];

#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
        _cx: opentelemetry::Context,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::CheckSteam {} => {
                respond!(msg, Response::CheckSteam(Ok(())));
                steam::check_steam_forums().await;
                Ok(())
            }
            Self::CheckReddit {} => {
                respond!(msg, Response::CheckReddit(Ok(())));
                reddit::check_reddit_findaunit().await;
                Ok(())
            }
            Self::PostReddit {} => {
                respond!(msg, Response::PostReddit(Ok(())));
                reddit::post_reddit_findaunit().await;
                Ok(())
            }
            Self::ReplyReddit { url } => {
                respond!(msg, Response::ReplyReddit(Ok(())));
                reddit::reply(msg, url).await;
                Ok(())
            }
        }
    }
}

#[allow(clippy::collapsible_match)]
pub async fn has_seen(url: String) -> bool {
    let req = events_request!(
        bootstrap::NC::get().await,
        synixe_events::recruiting::db,
        HasSeen { url }
    )
    .await;
    if let Ok(((ev, _), _)) = req {
        if let db::Response::HasSeen(seen) = ev {
            if let Ok(seen) = seen {
                return seen == Some(true);
            }
        }
    }
    false
}

pub async fn seen(url: String) {
    if let Err(e) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::recruiting::db,
        Seen { url }
    )
    .await
    {
        error!("failed marking url as seen: {}", e);
    }
}
