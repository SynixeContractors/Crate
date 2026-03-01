//! Events for the 2025 reset
/// Interact with the database
pub mod db {
    use serenity::model::prelude::UserId;
    use synixe_model::reset::UnclaimedKit;
    use synixe_proc::events_requests;
    use time::OffsetDateTime;
    use uuid::Uuid;
    events_requests!(db.reset {
        /// Get the unclaimed cert kits
        struct UnclaimedKits {
            /// The member to check
            member: UserId,
        } => (Result<Vec<UnclaimedKit>, String>)
        /// Can claim a cert kit
        struct CanClaim {
            /// The member to check
            member: UserId,
            /// The cert to check
            first_kit: Uuid,
        } => (Result<Option<Option<bool>>, String>)
        /// When the last kit was claimed
        struct LastClaim {
            /// The member to check
            member: UserId,
        } => (Result<Option<OffsetDateTime>, String>)
        /// Claime a cert kit
        struct ClaimKit {
            /// Member who claimed the kit
            member: UserId,
            /// Cert claimed
            first_kit: Uuid,
        } => (Result<(), String>)
    });
}
