use scraper::{Html, Selector};
use synixe_events::discord::write::{DiscordContent, DiscordMessage};
use synixe_proc::events_request_5;

use super::{
    candidate::{Candidate, Source},
    has_seen, seen, IGNORE, PING,
};

const STEAM_FORUM: &str = "https://steamcommunity.com/app/107410/discussions/21/";

#[allow(clippy::cognitive_complexity)]
pub async fn check_steam_forums() {
    debug!("Checking steam forums for new posts");

    let candidates = {
        let mut candidates = Vec::new();

        let selector_post: Selector = scraper::Selector::parse("a.forum_topic_overlay").unwrap();
        let selector_title: Selector = scraper::Selector::parse("div.topic").unwrap();
        let selector_content: Selector = scraper::Selector::parse(".forum_op .content").unwrap();

        let page = reqwest::get(STEAM_FORUM)
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
            if has_seen(url.clone()).await {
                continue;
            }
            let post = reqwest::get(&url).await.unwrap().text().await.unwrap();
            seen(url.clone()).await;
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
                    source: Source::Steam,
                    title,
                    link: url,
                    content,
                    ping,
                }
                .into(),
            );
        }
        candidates
    };
    for candidate in candidates {
        if let Err(e) = events_request_5!(
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
        .await
        {
            error!("Error sending candidate: {}", e);
        }
    }
}
