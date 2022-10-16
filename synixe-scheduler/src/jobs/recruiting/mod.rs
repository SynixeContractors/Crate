const IGNORE: [&str; 5] = ["exile", "vietnam", "police rp", "halo", "ww2"];
const PING: [&str; 2] = ["pmc", "persistent"];

mod candidate;
mod reddit;
mod steam;

pub use reddit::{check_reddit_findaunit, post_reddit_findaunit};
pub use steam::check_steam_forums;
use synixe_events::{
    parse_data,
    recruiting::db::{Request, Response},
    request,
};

#[allow(clippy::collapsible_match)]
pub async fn has_seen(url: String) -> bool {
    let req = request!(bootstrap::NC::get().await, Request::HasSeen { url }).await;
    if let Ok(msg) = req {
        let ((ev, _), _) = parse_data!(msg, Response);
        if let Response::HasSeen(seen) = ev {
            if let Ok(seen) = seen {
                return seen == Some(true);
            }
        }
    }
    false
}

pub async fn seen(url: String) {
    if let Err(e) = request!(bootstrap::NC::get().await, Request::Seen { url }).await {
        error!("failed marking url as seen: {}", e);
    }
}
