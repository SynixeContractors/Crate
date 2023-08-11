//! Docker containers

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

#[derive(Debug, Serialize, Deserialize)]
/// A container somewhere in the Synixe infrastructure
pub struct Container {
    /// Container ID
    id: String,
    /// Datacenter
    dc: String,
    /// Pretty name
    name: Option<String>,
}

impl Container {
    #[must_use]
    /// Create a new container
    pub const fn new(id: String, dc: String, name: Option<String>) -> Self {
        Self { id, dc, name }
    }

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

    #[must_use]
    /// Get the pretty name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[must_use]
    /// Get the key
    pub fn key(&self) -> String {
        format!("{}:{}", self.dc, self.id)
    }
}

#[derive(Debug, Serialize, Deserialize, Display, EnumString, IntoStaticStr, EnumIter)]
/// Adoplh server, Canada DC
pub enum Primary {
    /// Arma 3 - Main
    /// Used for all contracts, subcontracts, and specials
    #[strum(serialize = "Arma 3 - Contract")]
    Arma3Contracts,
    /// Arma 3 - Training
    /// Used for training
    #[strum(serialize = "Arma 3 - Training")]
    Arma3Training,
}

impl Primary {
    #[must_use]
    /// Get the container ID
    pub fn id(&self) -> String {
        match self {
            Self::Arma3Contracts => "arma-contracts".to_string(),
            Self::Arma3Training => "arma-training".to_string(),
        }
    }
}

impl From<Primary> for Container {
    fn from(container: Primary) -> Self {
        Self {
            id: container.id(),
            dc: String::from("monterey-primary"),
            name: Some(container.to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Display, EnumString, IntoStaticStr, EnumIter)]
/// Reynold Server, DO New York
pub enum Reynold {
    /// TeamSpeak 3
    /// Used for voice chat
    #[strum(serialize = "TeamSpeak 3")]
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
            name: Some(container.to_string()),
        }
    }
}
