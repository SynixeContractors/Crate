const IGNORE: [&str; 5] = ["exile", "vietnam", "police rp", "halo", "ww2"];
const PING: [&str; 2] = ["pmc", "persistent"];

mod candidate;
mod reddit;
mod steam;
mod store;

pub use reddit::check_reddit_findaunit;
pub use steam::check_steam_forums;
use store::Store;
