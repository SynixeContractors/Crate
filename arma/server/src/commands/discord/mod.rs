use arma_rs::Group;

mod member;
mod roles;

pub fn group() -> Group {
    Group::new()
        .group("member", member::group())
        .group("roles", roles::group())
}
