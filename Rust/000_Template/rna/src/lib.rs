mod app_config;
pub use self::app_config::AppConfig;

mod app;
pub use self::app::App;

mod core;
pub use self::core::Core;

mod math_ext;
pub use self::math_ext::*;

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
