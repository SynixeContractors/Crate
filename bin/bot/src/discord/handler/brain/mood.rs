//! Brodsky's mood, worked out from how the company did over the last week: money, reputation and missions

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
const REFRESH: Duration = Duration::from_mins(15);

// What counts as a notable week. Tune these to how the company usually does.
/// Net income in a week that's a big win, or as a loss a big loss
const BIG_WEEK: i64 = 200_000;
/// Reputation gained in a week that's a big gain, or as a loss a big loss. A mission is worth -200 to 150.
const BIG_REPUTATION: i64 = 7;

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
            Self::Cheerful => {
                "He is in a good mood: warm, playful, quick with a compliment wrapped in a joke."
            }
            Self::Content => "He is in his usual mood: dry, sarcastic, unbothered.",
            Self::Grumpy => {
                "He is grumpy: short-tempered, complains about money and lazy contractors, but still helps."
            }
            Self::Furious => {
                "He is furious: dramatic, threatening to sell the vehicles and go back to driving buses, blaming everyone. Still helps, grudgingly."
            }
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
    /// Scores the week on the three things he cares about: money, reputation and how missions went
    pub fn assess(activity: CompanyActivity) -> Self {
        let mut score = 0;
        let mut reasons = Vec::new();

        // Money
        if activity.balance < 0 {
            score -= 3;
            reasons.push(format!(
                "the company is in debt ({})",
                money(activity.balance)
            ));
        }
        let net = activity.income - activity.spending;
        if net >= BIG_WEEK {
            score += 2;
            reasons.push(format!("the company made {} this week", money(net)));
        } else if net > 0 {
            score += 1;
        } else if net <= -BIG_WEEK {
            score -= 2;
            reasons.push(format!("the company lost {} this week", money(-net)));
        } else if net < 0 {
            score -= 1;
        }

        // Reputation
        let reputation = activity.reputation;
        if reputation >= BIG_REPUTATION {
            score += 2;
            reasons.push(format!(
                "the company's reputation rose by {reputation} this week"
            ));
        } else if reputation > 0 {
            score += 1;
        } else if reputation <= -BIG_REPUTATION {
            score -= 2;
            reasons.push(format!(
                "the company's reputation fell by {} this week",
                -reputation
            ));
        } else if reputation < 0 {
            score -= 1;
        }

        // Missions: each success lifts him, each failure hits harder, partials don't move him
        let missions = (activity.successes - activity.failures * 2).clamp(-3, 2);
        score += i32::try_from(missions).unwrap_or(0);
        if activity.successes > 0 {
            reasons.push(format!(
                "{} mission{} succeeded this week",
                activity.successes,
                if activity.successes == 1 { "" } else { "s" }
            ));
        }
        if activity.partials > 0 {
            reasons.push(format!(
                "{} mission{} only partly succeeded this week",
                activity.partials,
                if activity.partials == 1 { "" } else { "s" }
            ));
        }
        if activity.failures > 0 {
            reasons.push(format!(
                "{} mission{} failed this week",
                activity.failures,
                if activity.failures == 1 { "" } else { "s" }
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

static CURRENT: LazyLock<Mutex<Option<(Instant, MoodReport)>>> = LazyLock::new(|| Mutex::new(None));

/// His mood right now, worked out again every `REFRESH`. `None` if the company's activity can't be
/// fetched and there's no earlier mood to fall back on.
pub async fn current() -> Option<MoodReport> {
    let mut current = CURRENT.lock().await;
    if let Some((at, report)) = current.as_ref()
        && at.elapsed() < REFRESH
    {
        return Some(report.clone());
    }
    if let Ok(Ok((Response::CompanyActivity(Ok(activity)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        CompanyActivity { days: DAYS }
    )
    .await
    {
        let report = MoodReport::assess(activity);
        *current = Some((Instant::now(), report.clone()));
        Some(report)
    } else {
        error!("failed to fetch company activity for brodsky's mood");
        current.as_ref().map(|(_, report)| report.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn week(net: i64, reputation: i64, successes: i64, failures: i64) -> CompanyActivity {
        CompanyActivity {
            balance: 100_000,
            income: 10_000 + net,
            spending: 10_000,
            reputation,
            successes,
            partials: 0,
            failures,
        }
    }

    #[test]
    fn quiet_week_is_content() {
        let report = MoodReport::assess(week(0, 0, 0, 0));
        assert_eq!(report.mood, Mood::Content);
        assert_eq!(report.reasons, Vec::<String>::new());
    }

    #[test]
    fn great_week_is_ecstatic() {
        let report = MoodReport::assess(week(40_000, 200, 2, 0));
        assert_eq!(report.mood, Mood::Ecstatic);
        assert_eq!(report.reasons.len(), 3);
    }

    #[test]
    fn failed_mission_is_grumpy() {
        let report = MoodReport::assess(week(0, 0, 0, 1));
        assert_eq!(report.mood, Mood::Grumpy);
        assert!(report.summary().contains("1 mission failed"));
    }

    #[test]
    fn failure_outweighs_success() {
        // A success and a failure in the same week still leaves him down
        let report = MoodReport::assess(week(0, 0, 1, 1));
        assert_eq!(report.mood, Mood::Grumpy);
    }

    #[test]
    fn reputation_counts() {
        assert_eq!(MoodReport::assess(week(0, 50, 0, 0)).mood, Mood::Cheerful);
        assert_eq!(MoodReport::assess(week(0, -300, 0, 0)).mood, Mood::Grumpy);
    }

    #[test]
    fn debt_is_furious() {
        let mut activity = week(-30_000, -200, 0, 1);
        activity.balance = -5_000;
        let report = MoodReport::assess(activity);
        assert_eq!(report.mood, Mood::Furious);
        assert!(report.summary().contains("in debt"));
    }
}
