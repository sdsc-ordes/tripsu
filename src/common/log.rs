use slog::{self, o, Drain};
use slog_async;
use std::{io, sync::Arc};

pub type Logger = slog::Logger;

#[allow(dead_code)]
fn no_out(_io: &mut dyn io::Write) -> io::Result<()> {
    return Ok(());
}

pub fn create_logger(use_stdout: bool) -> Arc<Logger> {
    let decorator = match use_stdout {
        false => slog_term::TermDecorator::new().stderr().build(),
        true => slog_term::TermDecorator::new().stdout().build(),
    };

    let drain = slog_term::FullFormat::new(decorator)
        .use_custom_timestamp(no_out)
        .build()
        .fuse();

    let drain = slog_async::Async::new(drain)
        .chan_size(5_000_000)
        .build()
        .fuse();

    return Arc::new(slog::Logger::root(drain, o!()));
}

/// Log trace level record
#[macro_export]
macro_rules! log_trace(
    ($log:expr, #$tag:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Trace, $tag, $($args)+)
    };
    ($log:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Trace, "", $($args)+)
    };
);

#[allow(unused_imports)]
pub use log_trace as trace;

/// Log debug level record
#[macro_export]
macro_rules! log_debug(
    ($log:expr, #$tag:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Debug, $tag, $($args)+)
    };
    ($log:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Debug, "", $($args)+)
    };
);

#[allow(unused_imports)]
pub use log_debug as debug;

/// Log info level record
#[macro_export]
macro_rules! log_info(
    ($log:expr, #$tag:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Info, $tag, $($args)+)
    };
    ($log:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Info, "", $($args)+)
    };
);

pub use log_info as info;

/// Log warn level record
#[macro_export]
macro_rules! log_warn(
    ($log:expr, #$tag:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Warning, $tag, $($args)+)
    };
    ($log:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Warning, "", $($args)+)
    };
);

#[allow(unused_imports)]
pub use log_warn as warn;

/// Log warn level record
#[macro_export]
macro_rules! log_error(
    ($log:expr, #$tag:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Error, $tag, $($args)+)
    };
    ($log:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Error, "", $($args)+)
    };
);

#[allow(unused_imports)]
pub use log_error as error;

#[macro_export]
macro_rules! log_critical(
    ($log:expr, #$tag:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Critical, $tag, $($args)+)
    };
    ($log:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Critical, "", $($args)+)
    };
);

#[allow(unused_imports)]
pub use log_critical as critical;

/// Log panic level record
#[macro_export]
macro_rules! log_panic(
    ($log:expr, #$tag:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Error, $tag, $($args)+);
        panic!();
    };
    ($log:expr, $($args:tt)+) => {
        slog::log!($log, slog::Level::Error, "", $($args)+);
        panic!();
    };
);

#[allow(unused_imports)]
pub use log_panic;
