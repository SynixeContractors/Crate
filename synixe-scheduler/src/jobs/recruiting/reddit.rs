use scraper::{Html, Selector};
use synixe_events::request;

use crate::jobs::recruiting::{
    candidate::{Candidate, Source},
    has_seen, seen, IGNORE, PING,
};

const REDDIT_FINDAUNIT: &str =
    "https://www.reddit.com/r/FindAUnit/new/?f=flair_name%3A%22Request%22";

pub async fn check_reddit_findaunit() {
    debug!("Checking reddit findaunit for new posts");

    let candidates = {
        let mut candidates = Vec::new();

        let selector_post: Selector =
            scraper::Selector::parse("a[data-click-id='comments']").unwrap();
        let selector_title: Selector = scraper::Selector::parse("h1").unwrap();
        let selector_content: Selector =
            scraper::Selector::parse("div[data-click-id='text']").unwrap();

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
            if has_seen(url.clone()).await {
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
        if let Err(e) = request!(
            bootstrap::NC::get().await,
            synixe_events::discord::write::Request::ChannelMessage {
                channel: synixe_meta::discord::channel::RECRUITING,
                message: synixe_events::discord::write::DiscordMessage::Embed(candidate)
            }
        )
        .await
        {
            error!("Error sending candidate: {}", e);
        }
    }
}
