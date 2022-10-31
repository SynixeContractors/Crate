//! Events for Certifications

/// Interact with the database.
pub mod db {
    use serenity::model::prelude::UserId;
    use synixe_model::certifications::{Certification, CertificationTrial};
    use synixe_proc::events_requests;
    use uuid::Uuid;

    events_requests!(db.certifications {
        /// List all certifications
        struct List {} => (Result<Vec<Certification>, String>)
        /// Certify a user
        struct Certify {
            /// Instructor
            instructor: UserId,
            /// The user to certify
            trainee: UserId,
            /// The certification to grant
            certification: Uuid,
            /// Notes about the certification
            notes: String,
            /// Did the user pass the certification?
            passed: bool,
        } => (Result<Option<CertificationTrial>, String>)
        /// Get all active certifications for a user
        struct Active {
            /// The user to check
            user: UserId,
        } => (Result<Vec<CertificationTrial>, String>)
    });
}
