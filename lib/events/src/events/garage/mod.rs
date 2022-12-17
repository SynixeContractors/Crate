//! Events for persistent gear

/// Interact with the database
pub mod db {
    use serenity::model::prelude::UserId;
    use synixe_model::garage::{ShopAsset, VehicleAsset};
    use synixe_proc::events_requests;

    events_requests!(db.garage {
        /// Get all vehicle assets in the garage
        struct FetchVehicleAssets {
            /// Filter the list of vehicles by stored status
            stored: Option<bool>,
            /// Search for a specific vehicle
            plate: Option<String>,
        } => (Result<Vec<VehicleAsset>, String>)
        /// Fetch a vehicle asset
        struct FetchVehicleAsset {
            /// The vehicle to fetch
            plate: String,
        } => (Result<Option<VehicleAsset>, String>)
        /// Get all vehicle assets in the shop
        struct FetchAllShopAssests {
            /// Search for a specific asset
            search: Option<String>,
        } => (Result<Vec<ShopAsset>, String>)
        /// Fetch a shop asset
        struct FetchShopAsset {
            /// The asset to fetch
            asset: String
        } => (Result<Option<ShopAsset>, String>)
        /// Purchase a shop asset
        struct PurchaseVehicleAsset {
            /// The asset to purchase
            id: uuid::Uuid,
            /// name/plate of the asset
            plate: String,
            /// The member purchasing the asset
            member: UserId,
        } => (Result<(), String>)
    });
}
