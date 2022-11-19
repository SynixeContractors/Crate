//! Events for persistent gear

/// Interact with the database
pub mod db {
    use synixe_model::garage::{ShopAsset, VehicleAsset};
    use synixe_proc::events_requests;

    events_requests!(db.garage {
        /// Get all vehicle assets in the garage
        struct FetchVehicleAssets {
            /// Filter the list of vehicles by stored status
            stored: Option<bool>,
        } => (Result<Vec<VehicleAsset>, String>)
        /// Fetch a vehicle asset
        struct FetchVehicleAsset {
            /// The vehicle to fetch
            plate: String
        } => (Result<Option<VehicleAsset>, String>)
        /// Get all vehicle assets in the shop
        struct FetchAllShopAssests {
        } => (Result<Vec<ShopAsset>, String>)
    });
}
