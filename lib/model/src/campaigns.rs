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
/// Advanced Equipment state for a device object. Not scoped to a campaign, since a device such as
/// a laptop can be carried between missions.
pub struct Ae3Device {
    /// Object id the device was saved under
    pub id: Uuid,
    /// Object class
    pub class: String,
    /// Encoded device state
    pub state: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// A packed (inventory item) Advanced Equipment device.
///
/// AE3 keeps its own item <-> state mapping in server memory only, so this tracks which item
/// class a device is currently packed as, to allow it to be rebuilt at mission start and still
/// redeployed after a restart.
pub struct Ae3PackedItem {
    /// The inventory item class the device is currently packed as
    pub item_class: String,
    /// Device id
    pub device_id: Uuid,
    /// The device's saved object class, needed to recreate it
    pub class: String,
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
