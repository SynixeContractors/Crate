//! Events for Certifications

/// Interact with the database.
pub mod db {
    use serenity::model::prelude::UserId;
    use synixe_model::certifications::{Certification, CertificationFirstKit, CertificationTrial};
    use synixe_proc::events_requests;
    use uuid::Uuid;

    events_requests!(db.certifications {
        /// List all certifications
        struct List {} => (Result<Vec<Certification>, String>)
        /// List all certifications that a member is certified to instruct
        struct ListInstructor {
            /// The member to list certifications for
            member: UserId,
        } => (Result<Vec<Certification>, String>)
        /// Get the name of a certification
        struct Name {
            /// The certification to get the name of
            certification: Uuid,
        } => (Result<Option<String>, String>)
        /// Certify a member
        struct Certify {
            /// Instructor
            instructor: UserId,
            /// The member to certify
            trainee: UserId,
            /// The certification to grant
            certification: Uuid,
            /// Notes about the certification
            notes: String,
            /// Did the member pass the certification?
            passed: bool,
        } => (Result<Option<CertificationTrial>, String>)
        /// Get all active certifications for a member
        struct Active {
            /// The member to check
            member: UserId,
        } => (Result<Vec<CertificationTrial>, String>)
        /// Get all active certifications
        struct AllActive {} => (Result<Vec<CertificationTrial>, String>)
        /// Get all active certifications that are expiring soon
        struct AllExpiring {
            /// The number of days before expiry to check
            days: i8,
        } => (Result<Vec<CertificationTrial>, String>)
        /// Get the count of passed trials for a certification
        struct PassedCount {
            /// The certification to check
            certification: Uuid,
            /// The member to check
            member: UserId,
        } => (Result<Option<i64>, String>)
        struct FirstKits {
            /// certification to filter
            certification: Option<Uuid>,
        } => (Result<Vec<CertificationFirstKit>, String>)
        struct GiveFirstKit {
            /// The first kit to give
            first_kit: Uuid,
            /// The member to give the first kit to
            member: UserId,
        } => (Result<(), String>)
    });
}

/// Execute events.
pub mod executions {
    use synixe_proc::events_requests;

    events_requests!(executor.certifications {
        /// Update the certification list
        struct CheckExpiries {} => (Result<(), String>)
        /// Ensure everyone has the correct roles
        struct CheckRoles {} => (Result<(), String>)
    });
}

/// Inform services about certifications.
pub mod publish {
    use synixe_model::certifications::CertificationTrial;
    use synixe_proc::events_publish;

    events_publish!(publish.certifications {
        /// A trial has been submitted
        struct TrialSubmitted {
            /// The certification trial
            trial: CertificationTrial,
        }
        /// A trial will expire soon
        struct TrialExpiring {
            /// The certification trial
            trial: CertificationTrial,
            /// Whole days until expiry
            days: i8,
        }
    });
}
