use arma_rs::Group;

mod member;

pub fn group() -> Group {
    Group::new().group("member", member::group())
}

// #[cfg(test)]
// mod tests {
//     use std::time::Duration;

//     use arma_rs::Value;

//     #[test]
//     fn fetch_brett() {
//         std::env::set_var("CRATE_SERVER", "test_get_brett");
//         let ext = super::super::init().testing();
//         unsafe {
//             let (_, code) = ext.call(
//                 "discord:member:get",
//                 Some(vec![String::from("76561198076832016")]),
//             );
//             assert_eq!(code, 0);
//         }
//         assert!(ext
//             .callback_handler(
//                 |name, func, data| {
//                     if name == "crate_log" {
//                         println!("{func}: {}", data.unwrap());
//                         return arma_rs::Result::<(), ()>::Continue;
//                     }
//                     assert_eq!(name, "crate:discord:member");
//                     assert_eq!(func, "get:ok");
//                     let Value::Array(data) = data.unwrap() else {
//                         panic!("expected array");
//                     };
//                     assert_eq!(data[0], Value::String(String::from("307524009854107648")));
//                     let Value::Array(roles) = data[1].clone() else {
//                         panic!("expected array");
//                     };
//                     assert!(roles.contains(&Value::String(String::from("700888852142751815"))));
//                     arma_rs::Result::Ok(())
//                 },
//                 Duration::from_secs(10)
//             )
//             .is_ok());
//     }

//     #[test]
//     fn fetch_missing() {
//         std::env::set_var("CRATE_SERVER", "test_get_brett");
//         let ext = super::super::init().testing();
//         unsafe {
//             let (_, code) = ext.call("discord:member:get", Some(vec![String::from("0123")]));
//             assert_eq!(code, 0);
//         }
//         assert!(ext
//             .callback_handler(
//                 |name, func, data| {
//                     if name == "crate_log" {
//                         println!("{func}: {}", data.unwrap());
//                         return arma_rs::Result::<(), ()>::Continue;
//                     }
//                     assert_eq!(name, "crate:discord:member");
//                     assert_eq!(func, "get:needs_link");
//                     arma_rs::Result::Ok(())
//                 },
//                 Duration::from_secs(10)
//             )
//             .is_ok());
//     }
// }
