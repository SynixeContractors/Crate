use async_trait::async_trait;
use synixe_events::{
    parse_data,
    recruiting::{db, executions::Request},
    request,
};

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
        _msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
        _cx: opentelemetry::Context,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Request::CheckSteam {} => {
                steam::check_steam_forums().await;
                Ok(())
            }
            Request::CheckReddit {} => {
                reddit::check_reddit_findaunit().await;
                Ok(())
            }
            Request::PostReddit {} => {
                reddit::post_reddit_findaunit().await;
                Ok(())
            }
            Request::ReplyReddit {} => todo!(),
        }
    }
}

#[allow(clippy::collapsible_match)]
pub async fn has_seen(url: String) -> bool {
    let req = request!(bootstrap::NC::get().await, db::Request::HasSeen { url }).await;
    if let Ok(msg) = req {
        let ((ev, _), _) = parse_data!(msg, db::Response);
        if let db::Response::HasSeen(seen) = ev {
            if let Ok(seen) = seen {
                return seen == Some(true);
            }
        }
    }
    false
}

pub async fn seen(url: String) {
    if let Err(e) = request!(bootstrap::NC::get().await, db::Request::Seen { url }).await {
        error!("failed marking url as seen: {}", e);
    }
}
