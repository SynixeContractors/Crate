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
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::CheckSteam {} => {
                if let Err(e) = respond!(msg, Response::CheckSteam(Ok(()))).await {
                    error!("failed to respond for CheckSteam{:?}", e);
                }
                steam::check_steam_forums().await;
                Ok(())
            }
            Self::CheckReddit {} => {
                if let Err(e) = respond!(msg, Response::CheckReddit(Ok(()))).await {
                    error!("failed to respond for CheckReddit{:?}", e);
                }
                reddit::check_reddit_findaunit().await;
                Ok(())
            }
            Self::PostReddit {} => {
                if let Err(e) = respond!(msg, Response::PostReddit(Ok(()))).await {
                    error!("failed to respond for PostReddit{:?}", e);
                }
                reddit::post_reddit_findaunit().await;
                Ok(())
            }
            Self::ReplyReddit { url } => {
                if let Err(e) = respond!(msg, Response::ReplyReddit(Ok(()))).await {
                    error!("failed to respond for ReplyReddit{:?}", e);
                }
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
    if let Ok(Ok((ev, _))) = req {
        if let db::Response::HasSeen(seen) = ev {
            if let Ok(seen) = seen {
                return seen == Some(Some(true));
            }
        }
    }
    true
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
