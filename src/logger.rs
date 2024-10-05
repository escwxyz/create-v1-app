use anyhow::Result;
use console::{style, Emoji};
use log::{Level, LevelFilter, Metadata, Record};

struct ColoredLogger;

impl log::Log for ColoredLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_str = match record.level() {
                Level::Error => format!("{}{}", Emoji("🚨 ", ""), style("ERROR").red().bold()),
                Level::Warn => format!("{}{}", Emoji("🚧 ", ""), style("WARN ").yellow().bold()),
                Level::Info => format!("{}{}", Emoji("🚀 ", ""), style("INFO ").green().bold()),
                Level::Debug => format!("{}{}", Emoji("🔍 ", ""), style("DEBUG").blue().bold()),
                Level::Trace => format!("{}{}", Emoji("🔎 ", ""), style("TRACE").magenta().bold()),
            };

            println!("{} - {}", level_str, record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: ColoredLogger = ColoredLogger;

pub fn initialize_logger() -> Result<()> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .map_err(|e| anyhow::anyhow!(e))
}

#[allow(dead_code)]
pub fn log_info(message: &str) {
    log::info!("{}", message);
}
#[allow(dead_code)]
pub fn log_error(message: &str) {
    log::error!("{}", message);
}
#[allow(dead_code)]
pub fn log_warn(message: &str) {
    log::warn!("{}", message);
}
#[allow(dead_code)]
pub fn log_debug(message: &str) {
    log::debug!("{}", message);
}

#[allow(dead_code)]
pub fn log_trace(message: &str) {
    log::trace!("{}", message);
}
