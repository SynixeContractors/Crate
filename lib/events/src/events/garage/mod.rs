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
