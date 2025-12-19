//! Docker containers

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Deserialize,
    Serialize,
    IntoStaticStr,
    Display,
    EnumIter,
    EnumString,
)]
pub enum ArmaServer {
    Arma3Contracts,
    Arma3Training,
}

impl ArmaServer {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Arma3Contracts => "arma3-contracts",
            Self::Arma3Training => "arma3-training",
        }
    }

    #[must_use]
    pub fn is_contracts(server: &str) -> bool {
        server == Self::Arma3Contracts.as_str()
    }
}

impl TryFrom<String> for ArmaServer {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str() {
            "arma3-contracts" => Self::Arma3Contracts,
            "arma3-training" => Self::Arma3Training,
            _ => return Err(format!("Unknown server: {value}")),
        })
    }
}
