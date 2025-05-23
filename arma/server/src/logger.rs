use arma_rs::Context;
use log::{Level, LevelFilter, Metadata, Record};

struct ArmaLogger {
    context: Context,
}

impl log::Log for ArmaLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if let Err(e) = self.context.callback_data(
                "crate:log",
                record.target(),
                Some(vec![
                    format!("{}", record.level()).to_uppercase(),
                    format!("{}", record.args()),
                ]),
            ) {
                eprintln!("error sending log: {e:?}");
            }
        }
    }

    fn flush(&self) {}
}

pub fn init(context: Context) {
    let logger = Box::leak(Box::new(ArmaLogger { context }));
    if let Err(e) = log::set_logger(logger).map(|()| log::set_max_level(LevelFilter::Debug)) {
        error!("failed to initialize logger: {e}");
    }
}
