//! Gear, Bank, Shop, Locker

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Vehicle asset currently in the garage
pub struct VehicleAsset {
    /// The id of the vehicle
    pub id: uuid::Uuid,
    /// The plate of the vehicle
    pub plate: String,
    /// Whether the vehicle is stored
    pub stored: bool,
    /// The class of the vehicle
    pub class: String,
    /// name of the vehicle
    pub name: String,
}

impl VehicleAsset {
    /// Create a new vehicle
    pub fn new(plate: String, class: String, name: String) -> Self {
        Self {
            plate,
            stored: true,
            class,
            name,
            id: uuid::Uuid::new_v4(),
        }
    }

    #[must_use]
    /// Get the vehicle
    pub fn vehicle_plate(&self) -> &str {
        self.plate.as_str()
    }

    #[must_use]
    /// is the vehicle stored
    pub fn stored(&self) -> bool {
        self.stored
    }

    #[must_use]
    /// Get the class of the vehicle
    pub fn class(&self) -> &str {
        self.class.as_str()
    }

    #[must_use]
    /// Get the name of the vehicle
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
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
}

impl ShopAsset {
    /// Create a new shop asset
    pub fn new(name: String, cost: i32, class: String, base: Option<Uuid>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            cost,
            class,
            base,
        }
    }

    #[must_use]
    /// Get the name of the asset
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[must_use]
    /// Get the cost of the asset
    pub fn cost(&self) -> i32 {
        self.cost
    }

    #[must_use]
    /// Get the class of the asset
    pub fn class(&self) -> &str {
        self.class.as_str()
    }
}
