use arma_rs::Group;

mod loadout;

pub fn group() -> Group {
    Group::new().group("loadout", loadout::group())
}
