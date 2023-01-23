//! Docker containers

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr};

#[derive(Debug, Serialize, Deserialize)]
/// A container somewhere in the Synixe infrastructure
pub struct Container {
    /// Container ID
    id: String,
    /// Datacenter
    dc: String,
}

impl Container {
    #[must_use]
    /// Get the container ID
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    /// Get the datacenter
    pub fn dc(&self) -> &str {
        &self.dc
    }
}

#[derive(Debug, Serialize, Deserialize, Display, EnumString, IntoStaticStr)]
/// Adoplh server, Canada DC
pub enum Adolph {
    /// Arma 3 - Main
    /// Used for all contracts, subcontracts, and specials
    Arma3Main,
    /// Arma 3 - Training
    Arma3Training,
}

impl Adolph {
    #[must_use]
    /// Get the container ID
    pub fn id(&self) -> String {
        match self {
            Self::Arma3Main => "arma-main".to_string(),
            Self::Arma3Training => "arma-training".to_string(),
        }
    }
}

impl From<Adolph> for Container {
    fn from(container: Adolph) -> Self {
        Self {
            id: container.id(),
            dc: String::from("adolph"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Display, EnumString, IntoStaticStr)]
/// Reynold Server, DO New York
pub enum Reynold {
    /// TeamSpeak 3
    TeamSpeak,
}

impl Reynold {
    #[must_use]
    /// Get the container ID
    pub fn id(&self) -> String {
        match self {
            Self::TeamSpeak => "teamspeak".to_string(),
        }
    }
}

impl From<Reynold> for Container {
    fn from(container: Reynold) -> Self {
        Self {
            id: container.id(),
            dc: String::from("reynold"),
        }
    }
}
