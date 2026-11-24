//! Brodsky's mood, worked out from what the company has been up to over the last week

use std::{
    sync::LazyLock,
    time::{Duration, Instant},
};

use synixe_events::gear::db::{CompanyActivity, Response};
use synixe_proc::events_request_5;
use tokio::sync::Mutex;

/// The days of activity the mood is based on
const DAYS: i32 = 7;
/// How long a mood lasts before it's worked out again
const REFRESH: Duration = Duration::from_secs(15 * 60);

// What counts as a notable week. Tune these to how the company usually does.
/// Net income in a week that's a big win, or as a loss a big loss
const BIG_WEEK: i64 = 20_000;
/// Average contractors per mission that's a great turnout
const GOOD_TURNOUT: i64 = 12;
/// Average contractors per mission that's a poor one
const POOR_TURNOUT: i64 = 6;
/// Casino take in a week, either way, worth noticing
const CASINO_SWING: i64 = 2_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    Ecstatic,
    Cheerful,
    Content,
    Grumpy,
    Furious,
}

impl Mood {
    const fn from_score(score: i32) -> Self {
        match score {
            3.. => Self::Ecstatic,
            1..=2 => Self::Cheerful,
            0 => Self::Content,
            -2..=-1 => Self::Grumpy,
            _ => Self::Furious,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Ecstatic => "ecstatic",
            Self::Cheerful => "cheerful",
            Self::Content => "content",
            Self::Grumpy => "grumpy",
            Self::Furious => "furious",
        }
    }

    /// How the mood shows in what he says, for his prompt
    pub const fn behaviour(self) -> &'static str {
        match self {
            Self::Ecstatic => {
                "He is in a fantastic mood: generous with praise, toasting the contractors, talking about buying something ridiculous for the company. Still a smart ass."
            }
            Self::Cheerful => "He is in a good mood: warm, playful, quick with a compliment wrapped in a joke.",
            Self::Content => "He is in his usual mood: dry, sarcastic, unbothered.",
            Self::Grumpy => {
                "He is grumpy: short-tempered, complains about money and lazy contractors, but still helps."
            }
            Self::Furious => {
                "He is furious: dramatic, threatening to sell the vehicles and go back to driving buses, blaming everyone. Still helps, grudgingly."
            }
        }
    }

    /// Reactions he sometimes leaves on messages
    pub const fn emoji(self) -> &'static [&'static str] {
        match self {
            Self::Ecstatic => &["🍻", "🎉", "💰", "🥳"],
            Self::Cheerful => &["😄", "👍", "🍺", "😎"],
            Self::Content => &["🤨", "👀", "🚌", "☕"],
            Self::Grumpy => &["😒", "🙄", "💸", "😤"],
            Self::Furious => &["😡", "🤬", "💀", "🔥"],
        }
    }
}

/// A mood, why he's in it, and the week it's based on
#[derive(Clone, Debug)]
pub struct MoodReport {
    pub mood: Mood,
    pub reasons: Vec<String>,
    pub activity: CompanyActivity,
}

impl MoodReport {
    pub fn assess(activity: CompanyActivity) -> Self {
        let mut score = 0;
        let mut reasons = Vec::new();

        if activity.balance < 0 {
            score -= 3;
            reasons.push(format!(
                "the company is in debt ({})",
                money(activity.balance)
            ));
        }

        let net = activity.income - activity.spending;
        match net {
            n if n >= BIG_WEEK => {
                score += 2;
                reasons.push(format!("the company made {} this week", money(n)));
            }
            n if n > 0 => score += 1,
            n if n <= -BIG_WEEK => {
                score -= 2;
                reasons.push(format!("the company lost {} this week", money(-n)));
            }
            n if n < 0 => score -= 1,
            _ => {}
        }

        match activity.missions {
            0 => {
                score -= 1;
                reasons.push(String::from("there were no missions this week and he is bored"));
            }
            1 => {}
            missions => {
                score += 1;
                reasons.push(format!("it was a busy week with {missions} missions"));
            }
        }

        if activity.missions > 0 {
            let turnout = activity.contractors_paid / activity.missions;
            if turnout >= GOOD_TURNOUT {
                score += 1;
                reasons.push(format!("missions had great turnouts, about {turnout} contractors each"));
            } else if turnout < POOR_TURNOUT {
                score -= 1;
                reasons.push(format!("barely anyone showed up, about {turnout} contractors a mission"));
            }
        }

        if activity.casino_take >= CASINO_SWING {
            score += 1;
            reasons.push(format!(
                "the casino took {} from contractors this week",
                money(activity.casino_take)
            ));
        } else if activity.casino_take <= -CASINO_SWING {
            score -= 1;
            reasons.push(format!(
                "the casino paid out {} more than it took this week",
                money(-activity.casino_take)
            ));
        }

        Self {
            mood: Mood::from_score(score),
            reasons,
            activity,
        }
    }

    /// The mood and its reasons as a sentence, for his prompt and `/brodsky mood`
    pub fn summary(&self) -> String {
        if self.reasons.is_empty() {
            format!("Brodsky is {}.", self.mood.label())
        } else {
            format!(
                "Brodsky is {} because {}.",
                self.mood.label(),
                self.reasons.join(", and ")
            )
        }
    }
}

fn money(amount: i64) -> String {
    i32::try_from(amount).map_or_else(
        |_| format!("${amount}"),
        |amount| bootstrap::format::money(amount, false),
    )
}

static CURRENT: LazyLock<Mutex<Option<(Instant, MoodReport)>>> =
    LazyLock::new(|| Mutex::new(None));

/// His mood right now, worked out again every `REFRESH`. `None` if the company's activity can't be
/// fetched and there's no earlier mood to fall back on.
pub async fn current() -> Option<MoodReport> {
    let mut current = CURRENT.lock().await;
    if let Some((at, report)) = current.as_ref()
        && at.elapsed() < REFRESH
    {
        return Some(report.clone());
    }
    match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        CompanyActivity { days: DAYS }
    )
    .await
    {
        Ok(Ok((Response::CompanyActivity(Ok(activity)), _))) => {
            let report = MoodReport::assess(activity);
            *current = Some((Instant::now(), report.clone()));
            Some(report)
        }
        _ => {
            error!("failed to fetch company activity for brodsky's mood");
            current.as_ref().map(|(_, report)| report.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn week(income: i64, spending: i64, missions: i64, contractors_paid: i64) -> CompanyActivity {
        CompanyActivity {
            balance: 100_000,
            income,
            spending,
            missions,
            contractors_paid,
            casino_take: 0,
        }
    }

    #[test]
    fn ordinary_week_is_content() {
        // One mission, a decent turnout, broke even
        let report = MoodReport::assess(week(10_000, 10_000, 1, 8));
        assert_eq!(report.mood, Mood::Content);
        assert!(report.reasons.is_empty());
    }

    #[test]
    fn great_week_is_ecstatic() {
        let report = MoodReport::assess(week(60_000, 20_000, 2, 30));
        assert_eq!(report.mood, Mood::Ecstatic);
        assert_eq!(report.reasons.len(), 3);
    }

    #[test]
    fn debt_is_furious() {
        let mut activity = week(0, 30_000, 0, 0);
        activity.balance = -5_000;
        let report = MoodReport::assess(activity);
        assert_eq!(report.mood, Mood::Furious);
        assert!(report.summary().contains("in debt"));
    }

    #[test]
    fn quiet_week_is_grumpy() {
        let report = MoodReport::assess(week(0, 5_000, 0, 0));
        assert_eq!(report.mood, Mood::Grumpy);
        assert!(report.summary().contains("bored"));
    }

    #[test]
    fn casino_counts() {
        let mut activity = week(10_000, 10_000, 1, 8);
        activity.casino_take = 5_000;
        assert_eq!(MoodReport::assess(activity).mood, Mood::Cheerful);
        activity.casino_take = -5_000;
        assert_eq!(MoodReport::assess(activity).mood, Mood::Grumpy);
    }
}
