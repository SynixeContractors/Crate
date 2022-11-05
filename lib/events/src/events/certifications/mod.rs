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
        /// List all certifications that a member is certified to instruct
        struct ListInstructor {
            /// The member to list certifications for
            member: UserId
        } => (Result<Vec<Certification>, String>)
        /// Get the name of a certification
        struct Name {
            /// The certification to get the name of
            certification: Uuid
        } => (Result<String, String>)
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
    });
}
