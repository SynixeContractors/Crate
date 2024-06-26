//! Running missions

#![allow(clippy::use_self)] // serde false positive

use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serenity::model::prelude::{ChannelId, MessageId};
use time::OffsetDateTime;
use uuid::Uuid;

pub mod aar;
mod listing;
pub use listing::Listing;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// A mission object
pub struct Mission {
    /// Mission id
    pub id: String,
    /// Mission name
    pub name: String,
    /// Mission summary
    pub summary: String,
    /// Mission description
    pub briefing: serde_json::Value,
    #[cfg_attr(feature = "sqlx", sqlx(rename = "type"))]
    /// Mission type
    pub typ: MissionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Mission play count
    pub play_count: Option<i64>,
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "mission_type", rename_all = "lowercase")
)]
/// Mission type
pub enum MissionType {
    /// Standard PMC mission
    #[default]
    Contract,
    /// Non-standard mission
    SubContract,
    /// Training mission
    Training,
    /// Special mission
    Special,
    /// Other mission
    Other,
}

impl MissionType {
    #[must_use]
    /// The folder name for the mission type on GitHub
    pub const fn github_folder(&self) -> &str {
        match self {
            Self::Contract | Self::SubContract => "contracts",
            Self::Training => "company",
            Self::Special | Self::Other => "specials",
        }
    }
}

impl From<i32> for MissionType {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Contract,
            1 => Self::SubContract,
            2 => Self::Training,
            3 => Self::Special,
            _ => Self::Other,
        }
    }
}

impl From<MissionType> for i32 {
    fn from(value: MissionType) -> Self {
        match value {
            MissionType::Contract => 0,
            MissionType::SubContract => 1,
            MissionType::Training => 2,
            MissionType::Special => 3,
            MissionType::Other => 4,
        }
    }
}

impl Display for MissionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Contract => "Contract".to_string(),
                Self::SubContract => "Subcontract".to_string(),
                Self::Training => "Training".to_string(),
                Self::Special => "Special".to_string(),
                Self::Other => "Other".to_string(),
            }
        )
    }
}

impl std::str::FromStr for MissionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "contract" => Ok(Self::Contract),
            "subcontract" => Ok(Self::SubContract),
            "training" => Ok(Self::Training),
            "special" => Ok(Self::Special),
            "other" => Ok(Self::Other),
            _ => Err(format!("Invalid mission type: {s}")),
        }
    }
}

#[cfg(feature = "mission-schedule")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// A scheduled mission
pub struct ScheduledMission {
    /// Unique id
    pub id: Uuid,
    /// Mission id
    pub mission: String,
    /// Message in #schedule
    pub schedule_message_id: Option<String>,
    /// Start datetime
    pub start: OffsetDateTime,
    /// Mission name
    pub name: String,
    /// Mission summary
    pub summary: String,
    /// Mission briefing
    pub briefing: serde_json::Value,
    /// Mission type
    #[cfg_attr(feature = "sqlx", sqlx(rename = "type"))]
    pub typ: MissionType,
}

#[cfg(feature = "mission-schedule")]
impl ScheduledMission {
    #[must_use]
    /// Get the channel and message id for the schedule message
    pub fn message(&self) -> Option<(ChannelId, MessageId)> {
        if let Some(msg) = &self.schedule_message_id {
            let (channel, message) = msg.split_once(':')?;
            let channel = ChannelId::from(channel.parse::<u64>().ok()?);
            let message = MessageId::from(message.parse::<u64>().ok()?);
            Some((channel, message))
        } else {
            None
        }
    }

    #[must_use]
    /// Get the briefing as a [`HashMap`]
    ///
    /// # Panics
    /// If the briefing is not a valid [`HashMap`]
    pub fn briefing(&self) -> HashMap<String, String> {
        let briefing: Map<String, Value> =
            serde_json::from_value(self.briefing.clone()).expect("always a valid map");
        briefing
            .into_iter()
            .map(|(k, v)| (k, v.as_str().expect("a valid string").to_string()))
            .collect()
    }
}

#[cfg(feature = "mission-schedule")]
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "missions_schedule_rsvp_state", rename_all = "lowercase")
)]
/// Mission type
pub enum Rsvp {
    /// The user is attending
    #[default]
    Yes,
    /// The user may attend, or be late
    Maybe,
    /// The user is not attending
    No,
}

#[cfg(feature = "mission-schedule")]
impl Display for Rsvp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yes => write!(f, "Yes"),
            Self::Maybe => write!(f, "Maybe"),
            Self::No => write!(f, "No"),
        }
    }
}

#[cfg(feature = "mission-schedule")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// A scheduled mission RSVP
pub struct MissionRsvp {
    /// Schedule mission id
    pub scheduled: Uuid,
    /// User's discord id
    pub member: String,
    /// User's RSVP
    pub state: Rsvp,
    /// Extra details
    pub details: Option<String>,
}
