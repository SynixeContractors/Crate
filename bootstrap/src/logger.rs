use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

/// Initializes the logger.
///
/// # Panics
///
/// Panics if the logger can not be initialized.
pub fn init() {
    let args: Vec<_> = std::env::args().collect();
    TermLogger::init(
        if args.contains(&String::from("--debug")) {
            LevelFilter::Debug
        } else if args.contains(&String::from("--trace")) {
            LevelFilter::Trace
        } else {
            LevelFilter::Info
        },
        ConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}
