//! Roles in the Synixe Discord server.

use serenity::model::prelude::RoleId;

// Seniority
/// Recruit
pub const RECRUIT: RoleId = RoleId(709_889_442_730_475_561);
/// Junior Member
pub const JUNIOR: RoleId = RoleId(700_892_097_775_009_842);
/// Member
pub const MEMBER: RoleId = RoleId(700_954_650_026_704_906);
/// Staff
pub const STAFF: RoleId = RoleId(700_888_852_142_751_815);

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

// Activity
/// Member is active, recently played
pub const ACTIVE: RoleId = RoleId(782_509_207_328_260_096);
/// Member is inactive, hasn't played in a while
pub const INACTIVE: RoleId = RoleId(700_938_613_092_909_128);

// Missions
/// Member is a mission maker
pub const MISSION_MAKER: RoleId = RoleId(973_300_919_015_997_491);
/// Member is a mission reviewer
pub const MISSION_REVIEWER: RoleId = RoleId(1_020_252_253_287_886_858);

// Certifications - Generic
/// Generic - Grenadier
pub const CERT_GRENADIER: RoleId = RoleId(780_138_101_834_121_236);
/// Generic - Crew Served Weapons
pub const CERT_CSW: RoleId = RoleId(932_076_894_768_209_930);
/// Generic - Scopes
pub const CERT_SCOPES: RoleId = RoleId(932_078_647_718_850_590);
/// Generic - Personal Electronics
pub const CERT_ELECTRONICS: RoleId = RoleId(932_077_564_124_602_388);

/// Specialist - Marksman
pub const CERT_MARKSMAN: RoleId = RoleId(780_137_008_411_705_375);
/// Specialist - Medic
pub const CERT_MEDIC: RoleId = RoleId(780_136_967_677_411_389);
/// Specialist - Automatic Rifleman
pub const CERT_AUTORIFLEMAN: RoleId = RoleId(780_137_042_180_046_848);
/// Specialist - Medium Anti-Tank
pub const CERT_MAT: RoleId = RoleId(881_304_829_794_848_778);
/// Specialist - Pilot
pub const CERT_PILOT: RoleId = RoleId(928_981_006_063_644_683);
/// Specialist - UAV Operator
pub const CERT_UAV: RoleId = RoleId(928_981_141_678_075_915);
/// Specialist - Engineer
pub const CERT_ENGINEER: RoleId = RoleId(814_987_669_921_726_514);

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