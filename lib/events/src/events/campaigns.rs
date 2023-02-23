//! Events for Arma Campaigns

/// Interact with the database.
pub mod db {
    use synixe_model::campaigns::{Group, Marker, Object, Unit};
    use synixe_proc::events_requests;
    use uuid::Uuid;

    events_requests!(db.certifications {
        /// Retrieve all objects for a campaign
        struct Objects {
            /// The campaign to get objects for
            campaign: Uuid,
        } => (Result<Vec<Object>, String>)
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
    });
}
