//! Mods

use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time_tz::{timezones::db::america::NEW_YORK, OffsetDateTimeExt};

#[derive(Clone, Debug, Serialize, Deserialize)]
/// A workshop mod
pub struct Workshop {
    /// Workshop ID
    pub workshop_id: String,
    /// Name of the mod
    pub name: String,
    /// Last updated time
    pub updated_at: OffsetDateTime,
}

impl Workshop {
    #[must_use]
    /// Workshop ID
    pub fn workshop_id(&self) -> &str {
        &self.workshop_id
    }

    #[must_use]
    /// Name of the mod
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    /// Last updated time
    pub const fn updated_at(&self) -> OffsetDateTime {
        self.updated_at
    }

    #[must_use]
    /// Mod needs updating
    pub async fn check_mod(&self) -> bool {
        let url = format!("https://steamcommunity.com/sharedfiles/filedetails/changelog/{}", self.workshop_id());
        let post = reqwest::get(&url)
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let document = Html::parse_document(&post);
        let selector_headline = Selector::parse("div.changelog").unwrap();

        let regex = Regex::new(r"(?m)Update: (?P<day>\d{1,2}) (?P<month>Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)(?:, (?P<year>\d{4}))? @ (?P<hour>\d{1,2}):(?P<minute>\d{2})(?P<ampm>am|pm)").unwrap();
        let first = document
            .select(&selector_headline)
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();
        let captures = regex.captures(&first).unwrap();
        let day = captures
            .name("day")
            .unwrap()
            .as_str()
            .parse::<i32>()
            .unwrap();
        let month = match captures.name("month").unwrap().as_str() {
            "Jan" => 1,
            "Feb" => 2,
            "Mar" => 3,
            "Apr" => 4,
            "May" => 5,
            "Jun" => 6,
            "Jul" => 7,
            "Aug" => 8,
            "Sep" => 9,
            "Oct" => 10,
            "Nov" => 11,
            "Dec" => 12,
            _ => panic!("invalid month"),
        };
        let year = captures.name("year").map_or(
            OffsetDateTime::now_utc()
                .to_timezone(NEW_YORK)
                .date()
                .year(),
            |y| y.as_str().parse::<i32>().unwrap(),
        );
        let hour = captures
            .name("hour")
            .unwrap()
            .as_str()
            .parse::<i32>()
            .unwrap();
        let minute = captures
            .name("minute")
            .unwrap()
            .as_str()
            .parse::<i32>()
            .unwrap();
        let ampm = captures.name("ampm").unwrap().as_str();
        let hour = match ampm {
            "am" => {
                if hour == 12 {
                    0
                } else {
                    hour
                }
            }
            "pm" => {
                if hour == 12 {
                    hour
                } else {
                    hour + 12
                }
            }
            _ => panic!("invalid am/pm"),
        };

        println!("time:[{first}]");
        println!("day:[{day}]\nmonth:[{month}]\nyear:[{year}]\nhour:[{hour}]\nminute:[{minute}]");
        true
    }
}
