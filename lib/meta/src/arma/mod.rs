//! Meta data for Arma 3

use std::fmt::Display;

/// Arma 3 DLCs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DLC {
    /// Maps - 249861
    Maps,
    /// Tactical Guide - 249862
    TacticalGuide,
    /// Zeus - 275700
    Zeus,
    /// Karts - 288520
    Karts,
    /// Helicopters - 304380
    Helicopters,
    /// Bundle 1 - 304400
    Bundle1,
    /// Marksman - 332350
    Marksman,
    /// Apex - 395180
    Apex,
    /// Laws of War - 571710
    LawsOfWar,
    /// Jets - 601670
    Jets,
    /// Bundle 2 - 612480
    Bundle2,
    /// Malden - 639600
    Malden,
    /// Tac-Ops - 744950
    TacOps,
    /// Tanks - 798390
    Tanks,
    /// Contact - 1021790
    Contact,
    /// Global Mobilization - 1042220
    GlobalMobilization,
    /// Prairie Fire - 1227700
    PrairieFire,
    /// CSLA - 1294440
    CSLA,
    /// Art of War - 1325500
    ArtOfWar,
    /// Western Sahara - 1681170
    WesternSahara,
}

impl TryFrom<u32> for DLC {
    type Error = ();
    fn try_from(dlc: u32) -> Result<Self, Self::Error> {
        match dlc {
            249_861 => Ok(Self::Maps),
            249_862 => Ok(Self::TacticalGuide),
            275_700 => Ok(Self::Zeus),
            288_520 => Ok(Self::Karts),
            304_380 => Ok(Self::Helicopters),
            304_400 => Ok(Self::Bundle1),
            332_350 => Ok(Self::Marksman),
            395_180 => Ok(Self::Apex),
            571_710 => Ok(Self::LawsOfWar),
            601_670 => Ok(Self::Jets),
            612_480 => Ok(Self::Bundle2),
            639_600 => Ok(Self::Malden),
            744_950 => Ok(Self::TacOps),
            798_390 => Ok(Self::Tanks),
            1_021_790 => Ok(Self::Contact),
            1_042_220 => Ok(Self::GlobalMobilization),
            1_227_700 => Ok(Self::PrairieFire),
            1_294_440 => Ok(Self::CSLA),
            1_325_500 => Ok(Self::ArtOfWar),
            1_681_170 => Ok(Self::WesternSahara),
            _ => Err(()),
        }
    }
}

impl From<DLC> for u32 {
    fn from(dlc: DLC) -> Self {
        match dlc {
            DLC::Maps => 249_861,
            DLC::TacticalGuide => 249_862,
            DLC::Zeus => 275_700,
            DLC::Karts => 288_520,
            DLC::Helicopters => 304_380,
            DLC::Bundle1 => 304_400,
            DLC::Marksman => 332_350,
            DLC::Apex => 395_180,
            DLC::LawsOfWar => 571_710,
            DLC::Jets => 601_670,
            DLC::Bundle2 => 612_480,
            DLC::Malden => 639_600,
            DLC::TacOps => 744_950,
            DLC::Tanks => 798_390,
            DLC::Contact => 1_021_790,
            DLC::GlobalMobilization => 1_042_220,
            DLC::PrairieFire => 1_227_700,
            DLC::CSLA => 1_294_440,
            DLC::ArtOfWar => 1_325_500,
            DLC::WesternSahara => 1_681_170,
        }
    }
}

impl Display for DLC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Maps => "Maps".to_string(),
                Self::TacticalGuide => "Tactical Guide".to_string(),
                Self::Zeus => "Zeus".to_string(),
                Self::Karts => "Karts".to_string(),
                Self::Helicopters => "Helicopters".to_string(),
                Self::Bundle1 => "Bundle 1".to_string(),
                Self::Marksman => "Marksman".to_string(),
                Self::Apex => "Apex".to_string(),
                Self::LawsOfWar => "Laws of War".to_string(),
                Self::Jets => "Jets".to_string(),
                Self::Bundle2 => "Bundle 2".to_string(),
                Self::Malden => "Malden".to_string(),
                Self::TacOps => "Tac-Ops".to_string(),
                Self::Tanks => "Tanks".to_string(),
                Self::Contact => "Contact".to_string(),
                Self::GlobalMobilization => "Global Mobilization".to_string(),
                Self::PrairieFire => "S.O.G. Prairie Fire".to_string(),
                Self::CSLA => "CSLA Iron Curtain".to_string(),
                Self::ArtOfWar => "Art of War".to_string(),
                Self::WesternSahara => "Western Sahara".to_string(),
            }
        )
    }
}
