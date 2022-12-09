//! Running missions

#![allow(clippy::use_self)] // serde false positive

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

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
    pub description: String,
    /// Mission type
    #[cfg_attr(feature = "sqlx", sqlx(rename = "type"))]
    pub typ: MissionType,
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "mission_type", rename_all = "lowercase")
)]
/// Mission type
pub enum MissionType {
    /// Standard Friday & Saturday mission
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

impl ToString for MissionType {
    fn to_string(&self) -> String {
        match self {
            Self::Contract => "Contract".to_string(),
            Self::SubContract => "SubContract".to_string(),
            Self::Training => "Training".to_string(),
            Self::Special => "Special".to_string(),
            Self::Other => "Other".to_string(),
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// A scheduled mission RSVP
pub struct MissionRsvp {
    /// Mission id
    pub mission: String,
    /// User's discord id
    pub member: String,
    /// User's RSVP
    pub state: Rsvp,
    /// Extra details
    pub details: Option<String>,
}
