//! Information about the Synixe Discord server.

use serenity::model::prelude::{GuildId, UserId};

pub mod channel;
pub mod role;

/// Synixe Contractors Guild ID.
pub const GUILD: GuildId = GuildId::new(700_888_247_928_356_905);

/// Brodsky's user ID.
pub const BRODSKY: UserId = UserId::new(1_028_418_063_168_708_638);
