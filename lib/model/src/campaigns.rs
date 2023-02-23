//! Campaigns

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// An non-unit object
pub struct Object {
    /// Campaign id
    pub campaign: Uuid,
    /// Object id
    pub id: Uuid,
    /// Object class
    pub class: String,
    /// Object data
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// A group object
pub struct Group {
    /// Campaign id
    pub campaign: Uuid,
    /// Group id
    pub id: Uuid,
    /// Group data
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// A unit object
pub struct Unit {
    /// Campaign id
    pub campaign: Uuid,
    /// Unit id
    pub id: Uuid,
    /// Unit class
    pub class: String,
    /// Unit group
    pub group: Uuid,
    /// Unit data
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// A marker
pub struct Marker {
    /// Campaign id
    pub campaign: Uuid,
    /// Marker name
    pub name: String,
    /// Marker data
    pub data: serde_json::Value,
}
