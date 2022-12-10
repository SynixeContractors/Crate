#![deny(missing_docs, missing_debug_implementations)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

//! Internal library for event driven communication between services.

mod events;
mod macros;
mod wrapper;

pub use events::*;
pub use wrapper::Wrapper;

// Rexport
pub use serde_json;

/// The Event can be invoked to get a response.
pub trait Evokable {
    /// NATS path to invoke the event on.
    fn path() -> &'static str;
    /// NATS path to invoke the event on.
    fn self_path(&self) -> &'static str {
        Self::path()
    }
    /// NATS name of the event.
    fn name(&self) -> &'static str;
}

/// The Event can be published.
pub trait Publishable {
    /// NATS path to invoke the event on.
    fn path() -> &'static str;
    /// NATS path to invoke the event on.
    fn self_path(&self) -> &'static str {
        Self::path()
    }
    /// NATS name of the event.
    fn name(&self) -> &'static str;
}
