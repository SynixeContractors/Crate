//! Events for persistent gear

/// Interact with the database
pub mod db {
    use serenity::model::prelude::UserId;
    use synixe_model::garage::{ShopAsset, VehicleAsset};
    use synixe_proc::events_requests;
    use uuid::Uuid;

    events_requests!(db.garage {
        /// Get all vehicles in the garage
        struct FetchStoredVehicles {
            /// Filter the list of vehicles by stored status
            stored: Option<bool>,
            /// Search for a specific vehicle
            plate: Option<String>,
        } => (Result<Vec<VehicleAsset>, String>)
        /// Get a vehicle in the garage
        struct FetchStoredVehicle {
            /// The vehicle to fetch
            plate: String,
        } => (Result<Option<VehicleAsset>, String>)
        /// Get available addons in the garage
        struct FetchStoredAddons {
            /// vehicle the addon is applied to
            plate: String,
        } => (Result<Vec<ShopAsset>, String>)
        /// Get all vehicle assets in the shop
        struct FetchShopAssets {
            /// Search for a specific asset
            search: Option<String>,
        } => (Result<Vec<ShopAsset>, String>)
        /// Fetch a shop asset
        struct FetchShopAsset {
            /// The asset to fetch
            asset: String
        } => (Result<Option<ShopAsset>, String>)
        /// Purchase a Vehicle asset
        struct PurchaseShopAsset {
            /// The asset to purchase
            id: Uuid,
            /// name/plate of the asset
            plate: Option<String>,
            /// The member purchasing the asset
            member: UserId,
        } => (Result<(), String>)
        /// Attach an addon to a vehicle
        struct AttachAddon {
            /// The vehicle to attach the addon to
            plate: String,
            /// The addon to attach
            addon: Uuid,
            /// The member attaching the addon
            member: UserId,
        } => (Result<(), String>)
        /// Detach an addon from a vehicle
        struct DetachAddon {
            /// The vehicle to detach the addon from
            plate: String,
            /// The member detaching the addon
            member: UserId,
        } => (Result<(), String>)
    });
}

/// Interact with the Arma Server
pub mod arma {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use synixe_proc::events_requests;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    /// Can a vehicle be spawned?
    pub enum CanSpawn {
        /// Yes, the vehicle can be spawned
        Yes,
        /// There is no spawn area for this vehicle type
        NoSpawnArea,
        /// The spawn area is blocked
        AreaBlocked,
        /// There are no players in the server
        NoPlayers,
        /// There is no player near the spawn area
        NoPlayersNear,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    /// The type of vehicle
    pub enum VehicleType {
        /// Car, Tank, etc
        Land,
        /// Helicopter, Plane, etc
        Air,
        /// Boat, Submarine, etc
        Water,
    }

    events_requests!(arma.garage {
        /// Check if a vehicle can be spawned
        struct CanSpawn {
            /// The type of vehicle to spawn
            typ: VehicleType,
        } => (Result<CanSpawn, String>)
        /// Spawn a vehicle
        struct Spawn {
            /// The class to spawn
            class: String,
            /// The state of the vehicle
            state: HashMap<String, serde_json::Value>,
        } => (Result<(), String>)
    });
}
