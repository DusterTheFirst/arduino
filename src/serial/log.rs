//! An implementation for using the log library to log over serial
//!
//! **Requires the feature `usb_logging`**

use crate::millis;

use super::{ansi, ansi::Color, USBSerialWriter, SERIAL};
use ansi::{EscapeSequence, Style};
use core::fmt::Write;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

/// Logging configuration
///
/// Allows a user to specify certain configurations of the logging
/// system. By default, the max log level is the log level set at
/// compile time. See the [compile time filters](https://docs.rs/log/0.4.8/log/#compile-time-filters)
/// section for more information. We also enable logging for all targets.
/// Set the `filters` collection to specify log targets of interest.
///
/// If the default configuration is good for you, use `Default::default()`
/// as the argument to [`init`](fn.init.html).
pub struct LoggingConfig {
    /// The max log level
    ///
    /// By default, we select the static max level. Users may
    /// override this if they'd like to bypass the statically-assigned
    /// max level
    pub max_level: LevelFilter,
    /// A list of filtered targets to log.
    ///
    /// If set to an empty slice (default), the logger performs no
    /// filtering. Otherwise, we filter the specified targets by
    /// the accompanying log level. If there is no level, we default
    pub filters: &'static [(&'static str, Option<LevelFilter>)],
}

impl Default for LoggingConfig {
    fn default() -> LoggingConfig {
        LoggingConfig {
            max_level: ::log::STATIC_MAX_LEVEL,
            filters: &[],
        }
    }
}

/// A logger for use with the log crate that outputs its data out over serial
pub struct USBLogger {
    enabled: bool,
    filters: &'static [(&'static str, Option<LevelFilter>)],
}

static mut LOGGER: USBLogger = USBLogger::new();

impl USBLogger {
    pub(crate) const fn new() -> Self {
        USBLogger {
            enabled: false,
            filters: &[],
        }
    }

    /// Initialize the USBLogger for use with the log crate
    pub fn init(config: LoggingConfig) -> Result<(), SetLoggerError> {
        unsafe {
            LOGGER.enabled = true;
            LOGGER.filters = config.filters;

            log::set_logger(&LOGGER).map(|()| log::set_max_level(config.max_level))
        }
    }

    /// Returns true if the target is in the filter, else false if the target is
    /// not in the list of kept targets. If the filter collection is empty, return
    /// true.
    fn filtered(&self, metadata: &::log::Metadata) -> bool {
        if self.filters.is_empty() {
            true
        } else if let Some(idx) = self
            .filters
            .iter()
            .position(|&(target, _)| target == metadata.target())
        {
            let (_, lvl) = self.filters[idx];
            lvl.is_none() || lvl.filter(|lvl| metadata.level() <= *lvl).is_some()
        } else {
            false
        }
    }
}

impl Log for USBLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.enabled // We're enabled
                    && metadata.level() <= log::max_level() // The log level is appropriate
                    && self.filtered(metadata) // The target is in the filter list
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level = record.level();

            let level_color = match &level {
                Level::Error => Color::LightRed,
                Level::Warn => Color::LightYellow,
                Level::Info => Color::LightBlue,
                Level::Debug => Color::Magenta,
                Level::Trace => Color::LightBlack,
            };

            // FIXME: remove carriage return
            writeln!(
                USBSerialWriter {},
                "[{}{}{} {} {}]: {}\r",
                EscapeSequence::new().set_fg(level_color),
                level,
                EscapeSequence::new().set_styles(&[Style::Clear]),
                record.target(),
                millis(),
                record.args()
            )
            .ok(); //FIXME: UNWRAP
        }
    }

    fn flush(&self) {
        SERIAL::send_now();
    }
}
