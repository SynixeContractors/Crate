//! Running missions

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
