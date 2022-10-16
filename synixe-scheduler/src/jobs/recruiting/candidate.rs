use synixe_events::discord::write::DiscordEmbed;

pub enum Source {
    Steam,
    Reddit,
}

pub struct Candidate {
    pub source: Source,
    pub title: String,
    pub link: String,
    pub content: String,
    pub ping: bool,
}

impl From<Candidate> for DiscordEmbed {
    fn from(val: Candidate) -> Self {
        Self {
            title: Some(val.title),
            description: Some(if val.ping { "@here " } else { "" }.to_string() + &val.content),
            url: Some(format!("https://reddit.com{}", val.link)),
            colour: Some(serenity::utils::Colour(match val.source {
                Source::Steam => 0x0066_C0F4,
                Source::Reddit => 0x00FF_5700,
            })),
        }
    }
}
