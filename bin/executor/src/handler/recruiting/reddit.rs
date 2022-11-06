use std::time::Duration;

use nats::asynk::Message;
use opentelemetry::{
    trace::{FutureExt, Tracer},
    Context,
};
use roux::{Reddit, User};
use scraper::{Html, Selector};
use synixe_events::{
    discord::write::{DiscordContent, DiscordMessage},
    recruiting::executions::Response,
    respond,
};
use synixe_proc::events_request;

use super::{
    candidate::{Candidate, Source},
    has_seen, seen, IGNORE, PING,
};

const REDDIT_FINDAUNIT: &str =
    "https://www.reddit.com/r/FindAUnit/new/?f=flair_name%3A%22Request%22";

pub async fn check_reddit_findaunit(cx: Context) {
    debug!("Checking reddit findaunit for new posts");

    let candidates = {
        let mut candidates = Vec::new();

        let selector_post: Selector =
            scraper::Selector::parse("a[data-click-id='comments']").unwrap();
        let selector_title: Selector = scraper::Selector::parse("h1").unwrap();
        let selector_content: Selector =
            scraper::Selector::parse("div[data-click-id='text']").unwrap();

        let tracer = bootstrap::tracer!("executor");
        let _span = tracer.start_with_context("recruiting.reddit.fetch_findaunit", &cx);
        let page = reqwest::get(REDDIT_FINDAUNIT)
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let mut posts = Vec::new();

        for post in Html::parse_document(&page).select(&selector_post) {
            posts.push(post.value().attr("href").unwrap().to_string());
        }
        for url in posts {
            if has_seen(url.clone(), cx.clone()).await {
                continue;
            }
            let full_url = format!("https://reddit.com{}", url);
            let post = reqwest::get(&full_url).await.unwrap().text().await.unwrap();
            seen(url).await;
            let document = Html::parse_document(&post);
            let content = document
                .select(&selector_content)
                .next()
                .unwrap()
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_lowercase();
            let title = document
                .select(&selector_title)
                .next()
                .unwrap()
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_lowercase();

            if IGNORE
                .iter()
                .any(|x| content.contains(x) || title.contains(x))
            {
                continue;
            }
            let ping = PING
                .iter()
                .any(|x| content.contains(x) || title.contains(x));
            candidates.push(
                Candidate {
                    source: Source::Reddit,
                    title,
                    link: full_url,
                    content,
                    ping,
                }
                .into(),
            );
        }
        candidates
    };
    for candidate in candidates {
        if let Err(e) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::discord::write,
            ChannelMessage {
                channel: synixe_meta::discord::channel::RECRUITING,
                message: DiscordMessage {
                    content: DiscordContent::Embed(candidate),
                    reactions: Vec::new(),
                },
                thread: None,
            }
        )
        .with_context(cx.clone())
        .await
        {
            error!("Error sending candidate: {}", e);
        }
    }
}

pub async fn post_reddit_findaunit(cx: Context) {
    debug!("in executor post reddit");
    let client = Reddit::new(
        "Ctirad Brodsky (by /u/synixe)",
        &std::env::var("REDDIT_CLIENT_ID").expect("REDDIT_CLIENT_SECRET not set"),
        &std::env::var("REDDIT_CLIENT_SECRET").expect("REDDIT_CLIENT_SECRET not set"),
    )
    .username(&std::env::var("REDDIT_USERNAME").expect("REDDIT_USERNAME not set"))
    .password(&std::env::var("REDDIT_PASSWORD").expect("REDDIT_PASSWORD not set"))
    .login()
    .await;
    let Ok(client) = client else { return };
    match client.submit_richtext(
        "[A3][Recruiting][NA/SA/OCE/SEA][Semi-milsim][18+]- Synixe Contractors - PMC - Persistent Gear - Manage your own kit",
        std::str::from_utf8(crate::Assets::get("reddit-findaunit.md").unwrap().data.as_ref()).unwrap(),
        "findaunit"
    ).await {
        Ok(_) => {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let submitted = User::new(&std::env::var("REDDIT_USERNAME").expect("REDDIT_USERNAME not set")).submitted(None).await;
            let link = if let Ok(submissions) = submitted {
                let post = submissions.data.children.first().unwrap();
                format!("https://reddit.com/r/{}/comments/{}", post.data.subreddit, post.data.id)
            } else {
                error!("Error getting reddit submissions: {:?}", submitted);
                String::new()
            };
            if let Err(e) = events_request!(
                bootstrap::NC::get().await,
                synixe_events::discord::write,
                ChannelMessage
                {
                    channel: synixe_meta::discord::channel::RECRUITING,
                    message: DiscordMessage {
                        content: DiscordContent::Embed(Candidate {
                            source: Source::Reddit,
                            title: "Reddit Post Submitted".to_string(),
                            link,
                            content: String::new(),
                            ping: false,
                        }.into()),
                        reactions: Vec::new(),
                    },
                    thread: None,
                }
            )
            .with_context(cx)
            .await
            {
                error!("Error sending reddit post candidate: {}", e);
            }
        },
        Err(e) => error!("Failed to post to reddit findaunit: {}", e),
    }
}

pub async fn reply(msg: Message, url: &str, cx: Context) {
    let client = Reddit::new(
        "Ctirad Brodsky (by /u/synixe)",
        &std::env::var("REDDIT_CLIENT_ID").expect("REDDIT_CLIENT_SECRET not set"),
        &std::env::var("REDDIT_CLIENT_SECRET").expect("REDDIT_CLIENT_SECRET not set"),
    )
    .username(&std::env::var("REDDIT_USERNAME").expect("REDDIT_USERNAME not set"))
    .password(&std::env::var("REDDIT_PASSWORD").expect("REDDIT_PASSWORD not set"))
    .login()
    .await;
    let Ok(client) = client else { return };
    let comment_id = url
        .trim_start_matches("https://reddit.com/r/FindAUnit/comments/")
        .split_once('/')
        .unwrap()
        .0;
    debug!("Comment ID: {}", comment_id);
    match client
        .comment(
            std::str::from_utf8(crate::Assets::get("reddit-reply.md").unwrap().data.as_ref())
                .unwrap(),
            &format!("t3_{}", comment_id),
        )
        .await
    {
        Ok(response) => {
            debug!("response: ({:?}) {:?}", response.status(), response);
            if response.status().is_success() {
                if let Err(e) = respond!(msg, Response::ReplyReddit(Ok(())))
                    .with_context(cx.clone())
                    .await
                {
                    error!("Error sending response: {}", e);
                }
            } else {
                error!(
                    "Failed to post to reddit findaunit: ({}) {:?}",
                    response.status(),
                    response
                );
                if let Err(e) = respond!(msg, Response::ReplyReddit(Err(format!("{:?}", response))))
                    .with_context(cx.clone())
                    .await
                {
                    error!("Error sending response: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to post to reddit findaunit: {}", e);
            if let Err(e) = respond!(msg, Response::ReplyReddit(Err(e.to_string())))
                .with_context(cx.clone())
                .await
            {
                error!("Error sending response: {}", e);
            }
        }
    }
}
