use std::collections::HashMap;

use arma_rs::Group;

mod bodybag;
mod loadout;
mod shop;

pub fn group() -> Group {
    Group::new()
        .group("bodybag", bodybag::group())
        .group("loadout", loadout::group())
        .group("shop", shop::group())
}

fn clean_items(items: &mut HashMap<String, i32>) {
    items.remove("ItemRadio");
    items.remove("ItemRadioAcreFlagged");
    items.retain(|_, v| *v != 0);
}
