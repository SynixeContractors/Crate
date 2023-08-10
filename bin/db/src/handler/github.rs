use async_trait::async_trait;
use synixe_events::github::db::{Request, Response};

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::UserByGitHub { github } => fetch_one_and_respond!(
                msg,
                *db,
                cx,
                Response::UserByGitHub,
                "SELECT member as value FROM github_usernames WHERE github = $1",
                github,
            ),
            Self::UserByDiscord { discord } => fetch_one_and_respond!(
                msg,
                *db,
                cx,
                Response::UserByDiscord,
                "SELECT github as value FROM github_usernames WHERE member = $1",
                discord.to_string(),
            ),
            Self::Link { discord, github } => execute_and_respond!(
                msg,
                *db,
                cx,
                Response::Link,
                "INSERT INTO github_usernames (member, github) VALUES ($1, $2)",
                discord.to_string(),
                github,
            ),
        }
    }
}
