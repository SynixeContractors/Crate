use scraper::{Html, Selector};
use serenity::builder::CreateEmbed;

const STEAM_FORUM: &str = "https://steamcommunity.com/app/107410/discussions/21/";
const REDDIT_FINDAUNIT: &str =
    "https://www.reddit.com/r/FindAUnit/new/?f=flair_name%3A%22Request%22";

const IGNORE: [&str; 5] = ["exile", "vietnam", "police rp", "halo", "ww2"];
const PING: [&str; 2] = ["pmc", "persistent"];

pub enum Source {
    Steam,
    Reddit,
}

struct Candidate {
    source: Source,
    title: String,
    link: String,
    content: String,
    ping: bool,
}

impl From<Candidate> for CreateEmbed {
    fn from(val: Candidate) -> Self {
        let mut embed = Self::default();
        embed.title(val.title);
        embed.url(val.link);
        embed.description(if val.ping { "@here " } else { "" }.to_string() + &val.content);
        embed.color(match val.source {
            Source::Steam => 0x0066_C0F4,
            Source::Reddit => 0x00FF_5700,
        });
        embed
    }
}

pub async fn check_steam_forums() {
    println!("Checking steam forums for new posts");

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
    for post in posts {
        let post = reqwest::get(post).await.unwrap().text().await.unwrap();
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
        if PING
            .iter()
            .any(|x| content.contains(x) || title.contains(x))
        {
            println!("Ping");
        }
        println!("{}: {}", title, content);
    }
}

pub async fn check_reddit_findaunit() {
    println!("Checking reddit findaunit for new posts");

    let selector_post: Selector = scraper::Selector::parse("a[data-click-id='comments']").unwrap();
    let selector_title: Selector = scraper::Selector::parse("h1").unwrap();
    let selector_content: Selector = scraper::Selector::parse("div[data-click-id='text']").unwrap();

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
    for post in posts {
        let post = reqwest::get(format!("https://reddit.com{}", post))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
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
        if PING
            .iter()
            .any(|x| content.contains(x) || title.contains(x))
        {
            println!("Ping");
        }
        println!("{}: {}", title, content);
    }
}
