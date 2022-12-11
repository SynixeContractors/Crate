/// Initializes the logger.
///
/// # Panics
///
/// Panics if the logger can not be initialized.
pub fn init() {
    tracing_subscriber::fmt::init();
}
