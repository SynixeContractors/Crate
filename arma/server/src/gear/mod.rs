use arma_rs::Group;

mod loadout;
mod shop;

pub fn group() -> Group {
    Group::new()
        .group("loadout", loadout::group())
        .group("shop", shop::group())
}
