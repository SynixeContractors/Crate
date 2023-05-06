use async_trait::async_trait;
use synixe_events::{
    discord::write::{DiscordContent, DiscordMessage},
    reputation::db::{Request, Response},
};
use synixe_proc::events_request_2;

use super::Handler;

#[async_trait]
#[allow(clippy::too_many_lines)]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::FriendlyShot {
                member,
                target,
                weapon,
            } => {
                log(format!(
                    "**Friendly Fire**\n<@{member}> shot {target} with {weapon}",
                ));
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::FriendlyShot,
                    "INSERT INTO reputation_events (member, event, reputation, data) VALUES ($1, 'friendly_fire', -1, $2)",
                    member.to_string(),
                    serde_json::json!({
                        "target": target.to_string(),
                        "weapon": weapon.to_string(),
                    }),
                )
            }
            Self::CivilianShot {
                member,
                target,
                weapon,
            } => {
                log(format!(
                    "**Civilian Shot**\n<@{member}> shot {target} with {weapon}",
                ));
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::CivilianShot,
                    "INSERT INTO reputation_events (member, event, reputation, data) VALUES ($1, 'civilian_shot', -2, $2)",
                    member.to_string(),
                    serde_json::json!({
                        "target": target.to_string(),
                        "weapon": weapon.to_string(),
                    }),
                )
            }
            Self::UnarmedShot {
                member,
                target,
                weapon,
            } => {
                log(format!(
                    "**Unarmed Shot**\n<@{member}> shot {target} with {weapon}",
                ));
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::UnarmedShot,
                    "INSERT INTO reputation_events (member, event, reputation, data) VALUES ($1, 'unarmed_shot', -2, $2)",
                    member.to_string(),
                    serde_json::json!({
                        "target": target.to_string(),
                        "weapon": weapon.to_string(),
                    }),
                )
            }
            Self::SurrenderingShot {
                member,
                target,
                weapon,
            } => {
                log(format!(
                    "**Surrendering Shot**\n<@{member}> shot {target} with {weapon}",
                ));
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::SurrenderingShot,
                    "INSERT INTO reputation_events (member, event, reputation, data) VALUES ($1, 'surrendering_shot', -2, $2)",
                    member.to_string(),
                    serde_json::json!({
                        "target": target.to_string(),
                        "weapon": weapon.to_string(),
                    }),
                )
            }
            Self::CaptiveShot {
                member,
                target,
                weapon,
            } => {
                log(format!(
                    "**Captive Shot**\n<@{member}> shot {target} with {weapon}",
                ));
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::CaptiveShot,
                    "INSERT INTO reputation_events (member, event, reputation, data) VALUES ($1, 'captive_shot', -4, $2)",
                    member.to_string(),
                    serde_json::json!({
                        "target": target.to_string(),
                        "weapon": weapon.to_string(),
                    }),
                )
            }
            Self::UnconsciousShot {
                member,
                target,
                weapon,
            } => {
                log(format!(
                    "**Unconscious Shot**\n<@{member}> shot {target} with {weapon}",
                ));
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::UnconsciousShot,
                    "INSERT INTO reputation_events (member, event, reputation, data) VALUES ($1, 'unconscious_shot', -1, $2)",
                    member.to_string(),
                    serde_json::json!({
                        "target": target.to_string(),
                        "weapon": weapon.to_string(),
                    }),
                )
            }
            Self::BuildingDamaged {
                member,
                target,
                weapon,
            } => {
                log(format!(
                    "**Building Damaged**\n<@{member}> damaged {target} with {weapon}",
                ));
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::BuildingDamaged,
                    "INSERT INTO reputation_events (member, event, reputation, data) VALUES ($1, 'building_damaged', -1, $2)",
                    member.to_string(),
                    serde_json::json!({
                        "target": target.to_string(),
                        "weapon": weapon.to_string(),
                    }),
                )
            }
            Self::FriendlyHealed { member, target } => {
                log(format!("**Friendly Healed**\n<@{member}> healed {target}",));
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::FriendlyHealed,
                    "INSERT INTO reputation_events (member, event, reputation, data) VALUES ($1, 'friendly_healed', 1, $2)",
                    member.to_string(),
                    serde_json::json!({
                        "target": target.to_string(),
                    }),
                )
            }
            Self::UnfriendlyHealed { member, target } => {
                log(format!(
                    "**Unfriendly Healed**\n<@{member}> healed {target}",
                ));
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::UnfriendlyHealed,
                    "INSERT INTO reputation_events (member, event, reputation, data) VALUES ($1, 'unfriendly_healed', 1, $2)",
                    member.to_string(),
                    serde_json::json!({
                        "target": target.to_string(),
                    }),
                )
            }
            Self::CivilianHealed { member, target } => {
                log(format!("**Civilian Healed**\n<@{member}> healed {target}",));
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::CivilianHealed,
                    "INSERT INTO reputation_events (member, event, reputation, data) VALUES ($1, 'civilian_healed', 2, $2)",
                    member.to_string(),
                    serde_json::json!({
                        "target": target.to_string(),
                    }),
                )
            }
            Self::MissionCompleted {
                member,
                mission,
                reputation,
            } => {
                log(format!(
                    "**Mission Completed**\n<@{member}> completed {mission} and earned {reputation} reputation",
                ));
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::MissionCompleted,
                    "INSERT INTO reputation_events (member, event, reputation, data) VALUES ($1, 'mission_completed', $2, $3)",
                    member.to_string(),
                    i32::from(*reputation),
                    serde_json::json!({
                        "mission": mission.to_string(),
                    }),
                )
            }
            Self::CurrentReputation { at } => {
                fetch_one_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::CurrentReputation,
                    "SELECT reputation($1) as value",
                    at,
                )
            }
            Self::UpdateReputation {
                member,
                reputation,
                reason,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::UpdateReputation,
                    "INSERT INTO reputation_events (member, event, reputation, data) VALUES ($1, 'staff', $2, $3)",
                    member.to_string(),
                    i32::from(*reputation),
                    serde_json::json!({
                        "reason": reason.to_string(),
                    }),
                )
            }
        }
    }
}

fn log(content: String) {
    tokio::spawn(async {
        events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::discord::write,
            GameAudit {
                message: DiscordMessage {
                    content: DiscordContent::Text(content),
                    reactions: Vec::new(),
                }
            }
        )
        .await
    });
}
