//! Scheduling and running missions.

#![allow(clippy::use_self)] // serde false positive

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
    pub typ: MissionType,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(rename_all = "lowercase"))]
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

impl From<u32> for MissionType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Contract,
            1 => Self::SubContract,
            2 => Self::Training,
            3 => Self::Special,
            _ => Self::Other,
        }
    }
}

/// Interact with the database.
pub mod db {
    use chrono::NaiveDateTime;
    use synixe_proc::events_requests;

    events_requests!(db.missions {
        /// Schedule a mission
        Schedule {
            /// The mission to schedule.
            mission: String,
            /// The day to schedule the mission.
            date: NaiveDateTime
        } => (Result<(), String>)
    });
}
