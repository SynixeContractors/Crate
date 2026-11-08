//! Events for Arma Campaigns

/// Interact with the database.
pub mod db {
    use synixe_model::campaigns::{Ae3Device, Ae3PackedItem, Group, Marker, Object, Unit};
    use synixe_proc::events_requests;
    use uuid::Uuid;

    events_requests!(db.campaigns {
        /// Retrieve all objects for a campaign
        struct Objects {
            /// The campaign to get objects for
            campaign: Uuid,
        } => (Result<Vec<Object>, String>)
        /// Retrieve the Advanced Equipment device states for a set of ids. Not scoped to a
        /// campaign; devices are looked up by their own id wherever they currently live.
        struct Ae3DevicesByIds {
            /// Device ids to look up
            ids: Vec<Uuid>,
        } => (Result<Vec<Ae3Device>, String>)
        /// Retrieve the Advanced Equipment device state for one device, not scoped to a campaign
        struct Ae3DeviceById {
            /// Device id
            id: Uuid,
        } => (Result<Vec<Ae3Device>, String>)
        /// Store an Advanced Equipment device state, not scoped to a campaign
        struct StoreAe3Device {
            /// Device id
            id: Uuid,
            /// Device class
            class: String,
            /// Encoded device state
            state: serde_json::Value,
        } => (Result<(), String>)
        /// Retrieve all packed (inventory item) Advanced Equipment devices, to rebuild AE3's
        /// in-memory item buffer at mission start
        struct Ae3PackedItems {} => (Result<Vec<Ae3PackedItem>, String>)
        /// Record that a device was packed into an inventory item
        struct StoreAe3PackedItem {
            /// The inventory item class the device was packed as
            item_class: String,
            /// Device id
            device_id: Uuid,
        } => (Result<(), String>)
        /// Record that a packed item was redeployed, so the mapping is no longer needed
        struct DeleteAe3PackedItem {
            /// The inventory item class that was redeployed
            item_class: String,
        } => (Result<(), String>)
        /// Retrieve all groups for a campaign
        struct Groups {
            /// The campaign to get groups for
            campaign: Uuid,
        } => (Result<Vec<Group>, String>)
        /// Retrieve all units for a campaign
        struct Units {
            /// The campaign to get units for
            campaign: Uuid,
        } => (Result<Vec<Unit>, String>)
        /// Retrieve all markers for a campaign
        struct Markers {
            /// The campaign to get markers for
            campaign: Uuid,
        } => (Result<Vec<Marker>, String>)
        /// Store an object
        struct StoreObject {
            /// The campaign to store the object for
            campaign: Uuid,
            /// Object id
            id: Uuid,
            /// Object class
            class: String,
            /// Object data
            data: serde_json::Value,
        } => (Result<(), String>)
        /// Store a group
        struct StoreGroup {
            /// The campaign to store the group for
            campaign: Uuid,
            /// Group id
            id: Uuid,
            /// Group data
            data: serde_json::Value,
        } => (Result<(), String>)
        /// Store a unit
        struct StoreUnit {
            /// The campaign to store the unit for
            campaign: Uuid,
            /// Unit id
            id: Uuid,
            /// Unit class
            class: String,
            /// Unit group
            group: Uuid,
            /// Unit data
            data: serde_json::Value,
        } => (Result<(), String>)
        /// Store a marker
        struct StoreMarker {
            /// The campaign to store the marker for
            campaign: Uuid,
            /// Marker name
            name: String,
            /// Marker data
            data: serde_json::Value,
        } => (Result<(), String>)
        /// Delete an object
        struct DeleteObject {
            /// The campaign to delete the object for
            campaign: Uuid,
            /// Object id
            id: Uuid,
        } => (Result<(), String>)
        /// Delete a group
        struct DeleteGroup {
            /// The campaign to delete the group for
            campaign: Uuid,
            /// Group id
            id: Uuid,
        } => (Result<(), String>)
        /// Delete a unit
        struct DeleteUnit {
            /// The campaign to delete the unit for
            campaign: Uuid,
            /// Unit id
            id: Uuid,
        } => (Result<(), String>)
        /// Delete a marker
        struct DeleteMarker {
            /// The campaign to delete the marker for
            campaign: Uuid,
            /// Marker name
            name: String,
        } => (Result<(), String>)
    });
}
