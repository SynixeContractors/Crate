//! Roles in the Synixe Discord server.

use serenity::model::prelude::RoleId;

// Seniority
/// Recruit
pub const RECRUIT: RoleId = RoleId::new(709_889_442_730_475_561);
/// Junior Member
pub const JUNIOR: RoleId = RoleId::new(700_892_097_775_009_842);
/// Member
pub const MEMBER: RoleId = RoleId::new(700_954_650_026_704_906);
/// Staff
pub const STAFF: RoleId = RoleId::new(700_888_852_142_751_815);

#[must_use]
/// Does a role meet or exceed the given seniority?
pub fn seniority_pass(role: RoleId, bar: RoleId) -> bool {
    if role == bar {
        return true;
    }
    match role {
        JUNIOR => bar == RECRUIT,
        MEMBER => bar == RECRUIT || bar == JUNIOR,
        STAFF => true,
        _ => false,
    }
}

// Special Access
/// Ability to use /docker
pub const DOCKER: RoleId = RoleId::new(1_066_950_087_441_399_919);
/// Key holder during voting
pub const KEY_HOLDER: RoleId = RoleId::new(1_306_713_960_690_749_481);
/// Leadership - Ability to use trusted commands
pub const LEADERSHIP: RoleId = RoleId::new(1_074_443_934_383_751_229);

// Activity
/// Member is active, recently played
pub const ACTIVE: RoleId = RoleId::new(782_509_207_328_260_096);
/// Member is inactive, hasn't played in a while
pub const INACTIVE: RoleId = RoleId::new(700_938_613_092_909_128);

// Missions
/// Member is a mission maker
pub const MISSION_MAKER: RoleId = RoleId::new(973_300_919_015_997_491);
/// Member is a mission reviewer
pub const MISSION_REVIEWER: RoleId = RoleId::new(1_020_252_253_287_886_858);

// Certifications - Generic
/// Generic - Grenades
pub const CERT_GRENADES: RoleId = RoleId::new(1_038_399_590_224_887_899);
/// Generic - LAT
pub const CERT_LAT: RoleId = RoleId::new(1_038_399_549_842_145_302);
/// Generic - Grenadier
pub const CERT_GRENADIER: RoleId = RoleId::new(780_138_101_834_121_236);
/// Generic - Crew Served Weapons
pub const CERT_CSW: RoleId = RoleId::new(932_076_894_768_209_930);
/// Generic - Scopes
pub const CERT_SCOPES: RoleId = RoleId::new(932_078_647_718_850_590);
/// Generic - Personal Electronics
pub const CERT_ELECTRONICS: RoleId = RoleId::new(932_077_564_124_602_388);

/// Specialist - Marksman
pub const CERT_MARKSMAN: RoleId = RoleId::new(780_137_008_411_705_375);
/// Specialist - Medic
pub const CERT_MEDIC: RoleId = RoleId::new(780_136_967_677_411_389);
/// Specialist - Automatic Rifleman
pub const CERT_AUTORIFLEMAN: RoleId = RoleId::new(780_137_042_180_046_848);
/// Specialist - Medium Anti-Tank
pub const CERT_MAT: RoleId = RoleId::new(881_304_829_794_848_778);
/// Specialist - Pilot
pub const CERT_PILOT: RoleId = RoleId::new(928_981_006_063_644_683);
/// Specialist - UAV Operator
pub const CERT_UAV: RoleId = RoleId::new(928_981_141_678_075_915);
/// Specialist - Engineer
pub const CERT_ENGINEER: RoleId = RoleId::new(814_987_669_921_726_514);
/// Specialist - Engi: Vehicle Logistics
pub const CERT_ENGI_VEHICLE_LOGISTICS: RoleId = RoleId::new(1_038_406_302_935_810_078);
/// Specialist - Engi: Explosives Handling
pub const CERT_ENGI_EXPLOSIVES: RoleId = RoleId::new(1_038_406_347_114_426_410);
/// Specialist - Engi: EOD
pub const CERT_ENGI_EOD: RoleId = RoleId::new(1_038_406_394_925_301_792);

#[cfg(test)]
mod tests {
    #[test]
    fn seniority() {
        use super::{seniority_pass, JUNIOR, MEMBER, RECRUIT, STAFF};

        assert!(seniority_pass(RECRUIT, RECRUIT));
        assert!(seniority_pass(JUNIOR, RECRUIT));
        assert!(seniority_pass(MEMBER, RECRUIT));
        assert!(seniority_pass(STAFF, RECRUIT));

        assert!(!seniority_pass(RECRUIT, JUNIOR));
        assert!(seniority_pass(JUNIOR, JUNIOR));
        assert!(seniority_pass(MEMBER, JUNIOR));
        assert!(seniority_pass(STAFF, JUNIOR));

        assert!(!seniority_pass(RECRUIT, MEMBER));
        assert!(!seniority_pass(JUNIOR, MEMBER));
        assert!(seniority_pass(MEMBER, MEMBER));
        assert!(seniority_pass(STAFF, MEMBER));

        assert!(!seniority_pass(RECRUIT, STAFF));
        assert!(!seniority_pass(JUNIOR, STAFF));
        assert!(!seniority_pass(MEMBER, STAFF));
        assert!(seniority_pass(STAFF, STAFF));
    }
}
