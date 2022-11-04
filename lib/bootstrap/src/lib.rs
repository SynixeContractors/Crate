#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

#[cfg(feature = "sqlx")]
mod db;
#[cfg(feature = "sqlx")]
pub use db::{DBPool, DB};

#[cfg(feature = "nats")]
mod nc;
#[cfg(feature = "nats")]
pub use nc::NC;

#[cfg(feature = "otlp")]
mod otlp;
#[cfg(feature = "otlp")]
pub use otlp::Lightstep;

pub mod logger;
