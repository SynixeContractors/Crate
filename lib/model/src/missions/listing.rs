use serde::{Deserialize, Serialize};

use super::Mission;

#[derive(Debug, Default, Serialize, Deserialize)]
/// Listing of all missions and maps
pub struct Listing {
    maps: Vec<String>,
    missions: Vec<Mission>,
}

impl Listing {
    #[must_use]
    /// Create a new listing
    pub const fn new() -> Self {
        Self {
            maps: Vec::new(),
            missions: Vec::new(),
        }
    }

    /// Add a map to the listing
    pub fn add_map(&mut self, map: String) {
        self.maps.push(map);
    }

    /// Add a mission to the listing
    pub fn add_mission(&mut self, mission: Mission) {
        self.missions.push(mission);
    }

    #[must_use]
    /// Get the maps
    pub fn maps(&self) -> &[String] {
        &self.maps
    }

    #[must_use]
    /// Get the missions
    pub fn missions(&self) -> &[Mission] {
        &self.missions
    }
}
