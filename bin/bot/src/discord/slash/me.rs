use std::{collections::HashMap, fmt::Write};

use serenity::{
    all::{
        CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType, UserId,
    },
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};
use synixe_events::{casino::db::GameStats, missions::db::MemberMission};
use synixe_meta::discord::channel::SCHEDULE;
use synixe_model::missions::Rsvp;
use synixe_proc::events_request_5;
use time::{Duration, OffsetDateTime};

use crate::discord::interaction::Interaction;

use super::AllowPublic;

pub fn register() -> CreateCommand {
    CreateCommand::new("me")
        .description("Everything about you at Synixe")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "overview",
                "Your money, missions, certifications and next mission",
            )
            .allow_public(),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "missions",
                "Your mission history and attendance",
            )
            .allow_public(),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "casino",
                "Your record at the casino",
            )
            .allow_public(),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        return Ok(());
    };
    let CommandDataOptionValue::SubCommand(options) = &subcommand.value else {
        return Ok(());
    };
    match subcommand.name.as_str() {
        "overview" => overview(ctx, command, options).await,
        "missions" => missions(ctx, command, options).await,
        "casino" => casino(ctx, command, options).await,
        _ => Ok(()),
    }
}

fn money(amount: impl Into<i64>) -> String {
    let amount = amount.into();
    i32::try_from(amount).map_or_else(
        |_| format!("${amount}"),
        |amount| bootstrap::format::money(amount, false),
    )
}

async fn member_missions(member: UserId) -> Option<Vec<MemberMission>> {
    match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        MemberMissions { member }
    )
    .await
    {
        Ok(Ok((synixe_events::missions::db::Response::MemberMissions(Ok(missions)), _))) => {
            Some(missions)
        }
        _ => None,
    }
}

#[allow(clippy::too_many_lines)]
async fn overview(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Opening your file...").await?;
    let member = command.user.id;
    let mut out = format!("## <@{member}>\n");

    // Money
    let balance = match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankBalance { member }
    )
    .await
    {
        Ok(Ok((synixe_events::gear::db::Response::BankBalance(Ok(balance)), _))) => {
            balance.unwrap_or(0)
        }
        _ => return interaction.reply("Failed to fetch your balance").await,
    };
    let locker = match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        LockerBalance { member }
    )
    .await
    {
        Ok(Ok((synixe_events::gear::db::Response::LockerBalance(Ok(locker)), _))) => locker,
        _ => 0,
    };
    let loadout = match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        LoadoutBalance { member }
    )
    .await
    {
        Ok(Ok((synixe_events::gear::db::Response::LoadoutBalance(Ok(loadout)), _))) => loadout,
        _ => 0,
    };
    let _ = writeln!(
        out,
        "**Cash** {} · **Locker** {} · **Loadout** {} · **Net worth** {}",
        money(balance),
        money(locker),
        money(loadout),
        money(i64::from(balance) + i64::from(locker) + i64::from(loadout)),
    );

    // Missions
    if let Some(missions) = member_missions(member).await {
        let stats = MissionStats::from(&missions);
        match missions.first() {
            Some(last) => {
                let _ = writeln!(
                    out,
                    "**Missions** {} · last was **{}** <t:{}:R>{}",
                    missions.len(),
                    last.name,
                    last.start.unix_timestamp(),
                    if stats.current_streak > 1 {
                        format!(" · {} week streak", stats.current_streak)
                    } else {
                        String::new()
                    }
                );
            }
            None => {
                let _ = writeln!(out, "**Missions** none yet, time to change that");
            }
        }
    }

    // Certifications
    let certifications = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        Active { member }
    )
    .await;
    let names = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        List {}
    )
    .await;
    if let (
        Ok(Ok((synixe_events::certifications::db::Response::Active(Ok(trials)), _))),
        Ok(Ok((synixe_events::certifications::db::Response::List(Ok(list)), _))),
    ) = (certifications, names)
    {
        let names = list
            .into_iter()
            .map(|cert| (cert.id, cert.name))
            .collect::<HashMap<_, _>>();
        let mut certs = trials
            .iter()
            .filter_map(|trial| {
                let name = names.get(&trial.certification)?;
                Some(trial.valid_until.map_or_else(
                    || name.clone(),
                    |until| format!("{name} (expires <t:{}:R>)", until.unix_timestamp()),
                ))
            })
            .collect::<Vec<_>>();
        certs.sort();
        let _ = writeln!(
            out,
            "**Certifications** {}",
            if certs.is_empty() {
                String::from("none")
            } else {
                certs.join(", ")
            }
        );
    }

    // Next mission
    if let Ok(Ok((synixe_events::missions::db::Response::UpcomingSchedule(Ok(mut upcoming)), _))) =
        events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            UpcomingSchedule {}
        )
        .await
    {
        upcoming.sort_by_key(|scheduled| scheduled.start);
        if let Some(next) = upcoming.first() {
            let rsvp = match events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                FetchMissionRsvps { scheduled: next.id }
            )
            .await
            {
                Ok(Ok((
                    synixe_events::missions::db::Response::FetchMissionRsvps(Ok(rsvps)),
                    _,
                ))) => {
                    match rsvps
                        .iter()
                        .find(|rsvp| rsvp.member == member.to_string())
                        .map(|rsvp| rsvp.state)
                    {
                        Some(Rsvp::Yes) => "you're attending",
                        Some(Rsvp::Maybe) => "you're a maybe",
                        Some(Rsvp::No) => "you declined",
                        None => "you haven't RSVP'd",
                    }
                }
                _ => "",
            };
            let _ = writeln!(
                out,
                "**Next mission** {} <t:{}:R> · {rsvp}",
                next.name,
                next.start.unix_timestamp()
            );
        }
    }

    // Notifications
    if let Ok(Ok((synixe_events::discord::db::Response::GetNotifications(Ok(notifications)), _))) =
        events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::discord::db,
            GetNotifications { member }
        )
        .await
    {
        let on = |on: bool| if on { "on" } else { "off" };
        let _ = writeln!(
            out,
            "-# Payday messages {} · mission reminders {} · change with `/notifications`",
            on(notifications.payday),
            on(notifications.mission_reminder)
        );
    }

    interaction.reply(out).await
}

/// Attendance worked out from the missions a member was paid for
#[derive(Debug, Default, PartialEq, Eq)]
pub struct MissionStats {
    /// Payouts across every mission
    pub earned: i64,
    /// Missions by type, most first
    pub by_type: Vec<(String, usize)>,
    /// Weeks in a row with a mission, up to this week or last week
    pub current_streak: usize,
    /// The most weeks in a row with a mission
    pub best_streak: usize,
}

impl MissionStats {
    pub fn from(missions: &[MemberMission]) -> Self {
        Self::at(missions, OffsetDateTime::now_utc())
    }

    /// As `from`, with "now" given so it can be tested
    fn at(missions: &[MemberMission], now: OffsetDateTime) -> Self {
        let mut by_type = HashMap::<String, usize>::new();
        for mission in missions {
            *by_type.entry(mission.typ.to_string()).or_default() += 1;
        }
        let mut by_type = by_type.into_iter().collect::<Vec<_>>();
        by_type.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        // Weeks are counted from the Unix epoch, a Thursday, which is fine for telling them apart
        let week = |at: OffsetDateTime| at.unix_timestamp().div_euclid(7 * 24 * 60 * 60);
        let mut weeks = missions.iter().map(|m| week(m.start)).collect::<Vec<_>>();
        weeks.sort_unstable();
        weeks.dedup();

        let mut best_streak = 0;
        let mut run = 0;
        let mut previous = None;
        for &w in &weeks {
            run = if previous == Some(w - 1) { run + 1 } else { 1 };
            best_streak = best_streak.max(run);
            previous = Some(w);
        }
        // `run` is the streak ending at the latest week, current if that's this week or last
        let current_streak = match weeks.last() {
            Some(&last) if week(now) - last <= 1 => run,
            _ => 0,
        };

        Self {
            earned: missions.iter().map(|m| i64::from(m.payout)).sum(),
            by_type,
            current_streak,
            best_streak,
        }
    }
}

/// The most recent missions listed
const RECENT_MISSIONS: usize = 10;

async fn missions(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Checking the logbook...").await?;
    let member = command.user.id;
    let Some(missions) = member_missions(member).await else {
        return interaction.reply("Failed to fetch your missions").await;
    };
    let (Some(last), Some(first)) = (missions.first(), missions.last()) else {
        return interaction
            .reply(format!(
                "You haven't been paid for a mission yet. Check <#{SCHEDULE}> for what's coming up!"
            ))
            .await;
    };
    let stats = MissionStats::from(&missions);

    let mut out = format!(
        "## <@{member}>'s missions\n**{}** missions since <t:{}:D>, earning {}\n",
        missions.len(),
        first.start.unix_timestamp(),
        money(stats.earned),
    );
    let _ = writeln!(
        out,
        "{}",
        stats
            .by_type
            .iter()
            .map(|(typ, count)| format!("{typ}: {count}"))
            .collect::<Vec<_>>()
            .join(" · ")
    );
    let _ = writeln!(
        out,
        "**Streak** {} week{} · best {}",
        stats.current_streak,
        if stats.current_streak == 1 { "" } else { "s" },
        stats.best_streak
    );
    if OffsetDateTime::now_utc() - last.start > Duration::days(30) {
        let _ = writeln!(out, "-# It's been a while, Brodsky misses you");
    }
    let _ = writeln!(out, "\n**Recent**");
    for mission in missions.iter().take(RECENT_MISSIONS) {
        let _ = writeln!(
            out,
            "<t:{}:d> **{}** · {}",
            mission.start.unix_timestamp(),
            mission.name,
            money(mission.payout)
        );
    }
    interaction.reply(out).await
}

async fn casino(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Asking the pit boss...").await?;
    let member = command.user.id;
    let Ok(Ok((synixe_events::casino::db::Response::History(Ok(games)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::casino::db,
        History { member }
    )
    .await
    else {
        return interaction
            .reply("Failed to fetch your casino history")
            .await;
    };
    if games.is_empty() {
        return interaction
            .reply("You've never set foot in the casino. Smart, or scared?")
            .await;
    }
    interaction.reply(casino_report(member, &games)).await
}

fn casino_report(member: UserId, games: &[GameStats]) -> String {
    let mut table = format!(
        "{:<12} {:>5} {:>6} {:>10} {:>10} {:>10}\n",
        "Game", "Bets", "Won", "Wagered", "Net", "Best"
    );
    let mut total = GameStats {
        game: String::from("Total"),
        bets: 0,
        wagered: 0,
        paid_out: 0,
        wins: 0,
        biggest_win: 0,
    };
    for game in games {
        total.bets += game.bets;
        total.wagered += game.wagered;
        total.paid_out += game.paid_out;
        total.wins += game.wins;
        total.biggest_win = total.biggest_win.max(game.biggest_win);
    }
    for game in games.iter().chain(std::iter::once(&total)) {
        let _ = writeln!(
            table,
            "{:<12} {:>5} {:>5}% {:>10} {:>10} {:>10}",
            game.game,
            game.bets,
            if game.bets == 0 {
                0
            } else {
                game.wins * 100 / game.bets
            },
            money(game.wagered),
            money(game.paid_out - game.wagered),
            money(game.biggest_win),
        );
    }
    let verdict = match total.paid_out - total.wagered {
        net if net > 0 => "The house hates you.",
        0 => "Perfectly even. Suspicious.",
        _ => "The house thanks you for your donation.",
    };
    format!("## <@{member}> at the casino\n```{table}```{verdict}")
}

#[cfg(test)]
mod tests {
    use synixe_model::missions::MissionType;
    use time::macros::datetime;
    use uuid::Uuid;

    use super::*;

    fn mission(start: OffsetDateTime, typ: MissionType, payout: i32) -> MemberMission {
        MemberMission {
            scheduled: Uuid::new_v4(),
            name: String::from("Test"),
            typ,
            start,
            payout,
        }
    }

    #[test]
    fn no_missions() {
        let stats = MissionStats::at(&[], datetime!(2026-11-24 12:00 UTC));
        assert_eq!(stats, MissionStats::default());
    }

    #[test]
    fn streaks() {
        let now = datetime!(2026-11-24 12:00 UTC);
        // Newest first, as MemberMissions returns them: three weeks running up to now, a gap, then two
        let missions = [
            mission(now - Duration::days(1), MissionType::Contract, 100),
            mission(now - Duration::days(2), MissionType::Contract, 100),
            mission(now - Duration::days(8), MissionType::SubContract, 50),
            mission(now - Duration::days(15), MissionType::Contract, 100),
            mission(now - Duration::days(40), MissionType::Contract, 100),
            mission(now - Duration::days(47), MissionType::Training, 0),
        ];
        let stats = MissionStats::at(&missions, now);
        assert_eq!(stats.current_streak, 3);
        assert_eq!(stats.best_streak, 3);
        assert_eq!(stats.earned, 450);
        assert_eq!(
            stats.by_type.first(),
            Some(&(MissionType::Contract.to_string(), 4))
        );
    }

    #[test]
    fn streak_lapses() {
        let now = datetime!(2026-11-24 12:00 UTC);
        let missions = [
            mission(now - Duration::days(20), MissionType::Contract, 100),
            mission(now - Duration::days(27), MissionType::Contract, 100),
        ];
        let stats = MissionStats::at(&missions, now);
        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.best_streak, 2);
    }

    #[test]
    fn casino_totals() {
        let games = [
            GameStats {
                game: String::from("coinflip"),
                bets: 4,
                wagered: 400,
                paid_out: 200,
                wins: 1,
                biggest_win: 200,
            },
            GameStats {
                game: String::from("lucky3"),
                bets: 1,
                wagered: 100,
                paid_out: 3000,
                wins: 1,
                biggest_win: 3000,
            },
        ];
        let report = casino_report(UserId::new(1), &games);
        assert!(report.contains("The house hates you."));
        assert!(report.contains("Total"));
    }
}
