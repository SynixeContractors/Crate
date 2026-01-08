//! Events for persistent gear

/// Interact with the database
pub mod db {
    use serde::{Deserialize, Serialize};
    use serenity::model::prelude::UserId;
    use synixe_model::garage::{ShopAsset, VehicleAsset, VehicleColor};
    use synixe_proc::events_requests;
    use uuid::Uuid;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    /// An order to purchase an asset
    pub enum ShopOrder {
        /// Purchase a vehicle
        Vehicle {
            /// The asset to purchase
            id: Uuid,
            /// The color of the asset
            color: Option<String>,
            /// The member purchasing the asset
            member: UserId,
        },
        /// Purchase an addon
        Addon {
            /// The asset to purchase
            id: Uuid,
            /// The member purchasing the asset
            member: UserId,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// Information about a vehicle for spawning
    pub struct SpawnInfo {
        /// The vehicle to spawn
        pub class: Option<String>,
        /// The state of the vehicle
        pub state: Option<serde_json::Value>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// A fetched plate
    pub struct FetchedPlate {
        /// The plate
        pub plate: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum FetchAsset {
        ByName(String),
        ByClass(String),
    }

    events_requests!(db.garage {
        /// Get all plates
        struct FetchPlates {
            /// Filter the plates with a wildcard
            search: Option<String>,
        } => (Result<Vec<FetchedPlate>, String>)
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
            asset: FetchAsset,
        } => (Result<Option<ShopAsset>, String>)
        /// Fetch the color options for a vehicle
        struct FetchVehicleColors {
            /// The vehicle to fetch colors for
            id: Uuid,
        } => (Result<Vec<VehicleColor>, String>)
        /// Fetch vehicle information for spawning
        struct FetchVehicleInfo {
            /// The vehicle to fetch information for
            plate: String,
        } => (Result<Option<SpawnInfo>, String>)
        /// Purchase a Vehicle asset
        struct PurchaseShopAsset {
            /// The asset to purchase
            order: ShopOrder,
        } => (Result<Option<String>, String>)
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

        /// Retrieve a vehicle from the garage
        struct RetrieveVehicle {
            /// The vehicle to retrieve
            plate: String,
            /// The member retrieving the vehicle
            member: UserId,
        } => (Result<(), String>)
        /// Store a vehicle in the garage
        struct StoreVehicle {
            /// The vehicle to store
            plate: String,
            /// The state of the vehicle
            state: serde_json::Value,
            /// The member storing the vehicle
            member: UserId,
        } => (Result<(), String>)
    });
}

/// Interact with the Arma Server
pub mod arma {
    use std::str::FromStr;

    use serde::{Deserialize, Serialize};
    use synixe_proc::events_requests;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    /// Can a vehicle be spawned?
    pub enum SpawnResult {
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
    impl FromStr for SpawnResult {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "Yes" => Ok(Self::Yes),
                "NoSpawnArea" => Ok(Self::NoSpawnArea),
                "AreaBlocked" => Ok(Self::AreaBlocked),
                "NoPlayers" => Ok(Self::NoPlayers),
                "NoPlayersNear" => Ok(Self::NoPlayersNear),
                _ => Err(format!("Unknown CanSpawn: {s}")),
            }
        }
    }

    events_requests!(arma.garage {
        /// Spawn a vehicle
        struct Spawn {
            /// The class to spawn
            class: String,
            /// The plate of the vehicle
            plate: String,
            /// The state of the vehicle
            state: serde_json::Value,
        } => (Result<SpawnResult, String>)
    });
}
