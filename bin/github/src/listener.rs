use serenity::all::UserId;
use synixe_events::{
    discord::write::DiscordContent,
    github::publish::Publish,
    serde_json::{self, Value},
};
use synixe_meta::discord::{BRODSKY, channel::MISSION_MAKING};
use synixe_proc::events_request_5;

include!("../../../lib/common/listener.rs");

#[allow(clippy::cognitive_complexity)]
pub async fn start() {
    let nats = bootstrap::NC::get().await;

    let mut sub = nats
        .queue_subscribe("synixe.publish.>", String::from("synixe-github"))
        .await
        .expect("Failed to subscribe to synixe.publish.*");
    while let Some(msg) = sub.next().await {
        let nats = nats.clone();
        tokio::spawn(async move {
            synixe_events::listener!(
                msg.clone(),
                nats.clone(),
                synixe_events::github::publish::Publish,
            );
        });
    }
}

#[async_trait]
impl Listener for Publish {
    async fn listen(
        &self,
        _msg: bootstrap::async_nats::message::Message,
        _nats: bootstrap::async_nats::Client,
    ) -> Result<(), bootstrap::anyhow::Error> {
        match &self {
            Self::Hook { data } => {
                let Ok(data) = serde_json::from_str::<Value>(data) else {
                    error!("Invalid json");
                    return Ok(());
                };

                let action = data.get("action").and_then(|v| v.as_str()).unwrap_or("");

                let Some(pr) = data.get("pull_request") else {
                    // Check if this is a comment
                    if data.get("issue").is_some() {
                        // maybe this is a comment on a PR?
                        return handle_comment(&data, action).await;
                    }
                    error!("No pull request data found on {action} action");
                    return Ok(());
                };

                let number = pr
                    .get("number")
                    .and_then(synixe_events::serde_json::Value::as_u64)
                    .map_or(0, |n| i32::try_from(n).unwrap_or(0));

                if number == 0 {
                    info!("No pull request number found for action {action}");
                    return Ok(());
                }

                let title = pr.get("title").and_then(|v| v.as_str()).unwrap_or("");
                if title.is_empty() {
                    info!("No title found for pull request {}", number);
                    return Ok(());
                }

                match action {
                    "opened" => {
                        info!("Pull request {} opened", number);

                        let user = pr
                            .get("user")
                            .and_then(|v| v.get("login"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        if user.is_empty() {
                            info!("No user found for pull request {}", number);
                            return Ok(());
                        }

                        pr_message(
                            number,
                            title.to_string(),
                            format!(
                                "Pull request #{number} opened: [{title}](<{}>)",
                                pr.get("html_url").and_then(|v| v.as_str()).unwrap_or("#")
                            ),
                        )
                        .await;

                        let Some(user) = get_user_id(user).await else {
                            error!("Failed to get user by github: {}", user);
                            return Ok(());
                        };
                        add_user(number, title.to_string(), user, "Author".to_string()).await;
                    }
                    "review_requested" => {
                        info!("Review requested on pull request {}", number);
                        // requested_reviewres is an array, with each item having "login"
                        let requested_reviewers = pr
                            .get("requested_reviewers")
                            .and_then(|v| v.as_array())
                            .cloned()
                            .unwrap_or_default();
                        for reviewer in requested_reviewers {
                            let user = reviewer.get("login").and_then(|v| v.as_str()).unwrap_or("");
                            if user.is_empty() {
                                info!(
                                    "No user found for review request on pull request {}",
                                    number
                                );
                                continue;
                            }
                            let Some(user) = get_user_id(user).await else {
                                error!("Failed to get user by github: {}", user);
                                continue;
                            };
                            add_user(number, title.to_string(), user, "Reviewer".to_string()).await;
                        }
                    }
                    "submitted" => {
                        info!("Review submitted on pull request {}", number);

                        let Some(review) = data.get("review") else {
                            error!("No review data found for pull request {}", number);
                            return Ok(());
                        };

                        let state = review
                            .get("state")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_lowercase();

                        let reviewer = review
                            .get("user")
                            .and_then(|v| v.get("login"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let reviewer_id = if reviewer.is_empty() {
                            None
                        } else {
                            get_user_id(reviewer).await
                        };

                        let by = reviewer_id.map_or_else(
                            || {
                                if reviewer.is_empty() {
                                    String::new()
                                } else {
                                    format!(" by {reviewer}")
                                }
                            },
                            |reviewer_id| format!(" by <@{reviewer_id}>"),
                        );

                        pr_message(
                            number,
                            title.to_string(),
                            format!(
                                "Review submitted on pull request #{number}{by}: [{state}](<{}>)",
                                review
                                    .get("html_url")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("#")
                            ),
                        )
                        .await;
                    }
                    "closed" => {
                        info!("Pull request {} closed", number);

                        let merged = pr
                            .get("merged")
                            .and_then(synixe_events::serde_json::Value::as_bool)
                            .unwrap_or(false);

                        if merged {
                            pr_message(
                                number,
                                title.to_string(),
                                format!(
                                    ":tada: Pull request #{number} merged: [{title}](<{}>)",
                                    pr.get("html_url").and_then(|v| v.as_str()).unwrap_or("#")
                                ),
                            )
                            .await;
                        } else {
                            pr_message(
                                number,
                                title.to_string(),
                                format!(
                                    "Pull request #{number} closed without merge: [{title}](<{}>)",
                                    pr.get("html_url").and_then(|v| v.as_str()).unwrap_or("#")
                                ),
                            )
                            .await;
                        }
                    }
                    "" => {
                        info!("Action not specified for pull request {}", number);
                    }
                    _ => {
                        info!("Unknown action: {}", action);
                    }
                }
            }
        }
        Ok(())
    }
}

async fn handle_comment(data: &Value, action: &str) -> Result<(), bootstrap::anyhow::Error> {
    let Some(issue) = data.get("issue") else {
        error!("No issue data found on {action} action");
        return Ok(());
    };

    // Check if it has "pull_request" under issue
    if issue.get("pull_request").is_none() {
        info!(
            "Comment on issue {}, not a pull request, ignoring",
            issue
                .get("number")
                .and_then(synixe_events::serde_json::Value::as_u64)
                .unwrap_or(0)
        );
        return Ok(());
    }

    if action != "created" {
        info!("Comment action is {action}, not created, ignoring");
        return Ok(());
    }

    let number = issue
        .get("number")
        .and_then(synixe_events::serde_json::Value::as_u64)
        .map_or(0, |n| i32::try_from(n).unwrap_or(0));

    if number == 0 {
        info!("No issue number found for comment action");
        return Ok(());
    }

    let title = issue.get("title").and_then(|v| v.as_str()).unwrap_or("");
    if title.is_empty() {
        info!("No title found for issue {}", number);
        return Ok(());
    }

    let Some(comment) = data.get("comment") else {
        error!("No comment data found on comment action");
        return Ok(());
    };

    let user = comment
        .get("user")
        .and_then(|v| v.get("login"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if user.is_empty() {
        info!("No user found for comment on issue {}", number);
        return Ok(());
    }
    let user = get_user_id(user).await.unwrap_or_else(|| {
        error!("Failed to get user by github: {}", user);
        UserId::new(0)
    });

    pr_message(
        number,
        title.to_string(),
        format!(
            "Comment on pull request #{number} by <@{user}>: [Read comment](<{}>)",
            comment
                .get("html_url")
                .and_then(|v| v.as_str())
                .unwrap_or("#")
        ),
    )
    .await;

    Ok(())
}

async fn add_user(number: i32, title: String, user: UserId, reason: String) {
    if let Err(e) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::discord::write,
        PullRequestThreadUser {
            number,
            title,
            user,
            reason,
        }
    )
    .await
    {
        error!("Failed to save pull request thread: {}", e);
    }
}

async fn pr_message(number: i32, title: String, content: String) {
    if let Err(e) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::discord::write,
        PullRequestThreadMessage {
            number,
            title,
            content: DiscordContent::Text(content),
        }
    )
    .await
    {
        error!("Failed to save pull request thread: {}", e);
    }
}

async fn get_user_id(github: &str) -> Option<UserId> {
    if github == "Copilot" || github == "SynixeBrodsky" {
        return Some(BRODSKY);
    }
    let Ok(Ok((synixe_events::github::db::Response::UserByGitHub(Ok(Some(id))), _))) =
        events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::github::db,
            UserByGitHub {
                github: github.to_string(),
            }
        )
        .await
    else {
        error!("Failed to get user by github: {}", github);
        message_all(format!("I don't recognize the GitHub user `{github}`."));
        return None;
    };
    let Ok(id) = id.parse::<u64>() else {
        error!("Failed to parse user id: {}", id);
        return None;
    };
    Some(UserId::new(id))
}

fn message_all(message: String) {
    tokio::spawn(async {
        if let Err(e) = synixe_proc::events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::discord::write,
            ChannelMessage {
                channel: MISSION_MAKING,
                thread: None,
                message: synixe_events::discord::write::DiscordMessage {
                    content: synixe_events::discord::write::DiscordContent::Text(message),
                    reactions: vec![],
                }
            }
        )
        .await
        {
            error!("Failed to game audit: {}", e);
        }
    });
}
