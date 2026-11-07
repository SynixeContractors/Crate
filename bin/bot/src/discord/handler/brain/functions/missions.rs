use serde_json::json;
use serenity::prelude::Context;
use synixe_events::missions::db::Response;
use synixe_proc::events_request_5;
use time::format_description::well_known::Rfc3339;
use time_tz::{OffsetDateTimeExt, timezones::db::america::NEW_YORK};

use super::BrainFunction;

const LIMIT: usize = 5;

pub struct Upcoming {}

#[async_trait::async_trait]
impl BrainFunction for Upcoming {
    fn name(&self) -> &'static str {
        "upcoming_missions"
    }

    fn desc(&self) -> &'static str {
        "Get the details of the next upcoming missions (contracts and subcons): name, type, start time (New York time), summary, and briefing sections. Use this when asked about upcoming missions instead of guessing."
    }

    fn args(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn run(&self, _ctx: &Context, _args: serde_json::Value) -> Option<serde_json::Value> {
        let Ok(Ok((Response::UpcomingSchedule(Ok(mut upcoming)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            UpcomingSchedule {}
        )
        .await
        else {
            error!("failed to fetch upcoming missions for brain");
            return Some(json!({
                "error": "failed to fetch upcoming missions",
            }));
        };

        upcoming.sort_by_key(|m| m.start);
        let missions = upcoming
            .into_iter()
            .take(LIMIT)
            .map(|m| {
                json!({
                    "name": m.name,
                    "mission_id": m.mission,
                    "type": m.typ,
                    "start": m.start.to_timezone(NEW_YORK).format(&Rfc3339).unwrap_or_default(),
                    "summary": m.summary,
                    "briefing": m.briefing,
                })
            })
            .collect::<Vec<_>>();

        Some(json!({
            "upcoming_missions": missions,
        }))
    }
}
