use serde_json::json;
use serenity::prelude::Context;

use super::{BrainFunction, Caller};
use crate::discord::handler::brain::mood;

pub struct Finances {}

#[async_trait::async_trait]
impl BrainFunction for Finances {
    fn name(&self) -> &'static str {
        "company_week"
    }

    fn desc(&self) -> &'static str {
        "Get how the company did over the last week: its balance, income and spending, reputation change, and how missions went, and the mood it puts you in. Use it when asked how you are, how the company is doing, or why you're in the mood you're in."
    }

    fn args(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn run(
        &self,
        _ctx: &Context,
        _caller: &Caller,
        _args: serde_json::Value,
    ) -> Option<serde_json::Value> {
        let Some(report) = mood::current().await else {
            return Some(json!({ "error": "the books are unavailable right now" }));
        };
        let a = report.activity;
        Some(json!({
            "mood": report.mood.label(),
            "why": report.reasons,
            "balance": a.balance,
            "income_this_week": a.income,
            "spending_this_week": a.spending,
            "reputation_change_this_week": a.reputation,
            "missions_succeeded_this_week": a.successes,
            "missions_partly_succeeded_this_week": a.partials,
            "missions_failed_this_week": a.failures,
        }))
    }
}
