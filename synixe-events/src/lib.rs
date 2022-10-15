#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

//! Internal library for event driven communication between services.

mod events;
// mod macros;

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

// pub trait Publishable {
//     fn path() -> &'static str;
//     fn self_path(&self) -> &'static str {
//         Self::path()
//     }
//     fn name(&self) -> &'static str;
// }
