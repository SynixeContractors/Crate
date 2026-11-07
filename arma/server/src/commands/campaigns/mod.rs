use arma_rs::Group;

mod ae3;
mod groups;
mod markers;
mod objects;
mod units;

pub fn group() -> Group {
    Group::new()
        .group("ae3", ae3::group())
        .group("groups", groups::group())
        .group("markers", markers::group())
        .group("objects", objects::group())
        .group("units", units::group())
}
