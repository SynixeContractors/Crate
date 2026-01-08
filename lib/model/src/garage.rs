//! Gear, Bank, Shop, Locker

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Vehicle asset currently in the garage
pub struct VehicleAsset {
    /// The plate of the vehicle
    pub plate: String,
    /// name of the vehicle
    pub name: String,
    /// The id of the vehicle
    pub id: Uuid,
    /// The addon attached
    pub addon: Option<Uuid>,
    /// Whether the vehicle is stored
    pub stored: bool,
    /// The class of the vehicle
    pub class: String,
    /// The count of addons available for the vehicle
    pub addons: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Shop asset ready to be purchased
pub struct ShopAsset {
    /// The id of the asset
    pub id: Uuid,
    /// The name of the asset
    pub name: String,
    /// The price of the asset
    pub cost: i32,
    /// The class of asset
    pub class: String,
    /// if the asset can be attached
    pub base: Option<Uuid>,
    /// plate template
    pub plate_template: Option<String>,
    /// fuel capacity in litres
    pub fuel_capacity: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Vehicle color options
pub struct VehicleColor {
    /// The id of the asset
    pub id: Uuid,
    /// The name of the color
    pub name: String,
    /// The textures for the color
    pub texture_source: String,
}
