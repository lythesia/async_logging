extern crate chrono;

use std::fmt;
use std::fmt::Formatter;
use std::io::Error;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicU8, Arc};

mod async_logging;
mod buf;
mod count_down_latch;
mod macros;
mod writer;

pub struct AsyncLogger {
    logger: async_logging::AsyncLogging,
    level: Arc<AtomicU8>,
}

#[derive(Clone)]
pub enum Level {
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Level {
    fn from_u8(x: u8) -> Option<Self> {
        match x {
            1 => Some(Level::Error),
            2 => Some(Level::Warn),
            3 => Some(Level::Info),
            4 => Some(Level::Debug),
            5 => Some(Level::Trace),
            _ => None,
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        })
    }
}

impl AsyncLogger {
    pub fn new(
        basename: String,
        file_size: usize,
        flush_interval: u64,
    ) -> Result<AsyncLogger, Error> {
        Ok(AsyncLogger {
            logger: async_logging::AsyncLogging::new(basename, file_size, flush_interval)?,
            level: Arc::new(AtomicU8::from(Level::Info as u8)),
        })
    }

    pub fn start(&mut self) {
        self.logger.start();
    }

    pub fn stop(&mut self) {
        self.logger.stop();
    }

    pub fn set_log_level(&mut self, level: Level) {
        self.level.store(level as u8, Ordering::Relaxed);
    }

    pub fn get_log_level(&self) -> Level {
        Level::from_u8(self.level.load(Ordering::Relaxed)).unwrap()
    }

    pub fn log(&self, level: Level, msg: String) {
        if (level as u8) <= self.level.load(Ordering::Relaxed) {
            self.logger.append(msg);
        }
    }
}

#[doc(hidden)]
pub fn current_thread_name() -> String {
    match std::thread::current().name() {
        Some(name) => name.into(),
        _ => format!("{:?}", std::thread::current().id()),
    }
}

#[doc(hidden)]
pub fn now() -> String {
        chrono::Local::now().format("%Y/%m/%d-%H:%M:%S%.6f").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASENAME: &str = "/tmp/async_logger";
    const FILE_SIZE: usize = 4096;

    #[test]
    fn test_logger_simple() {
        let mut logger =
            AsyncLogger::new(BASENAME.into(), FILE_SIZE, 2).expect("failed to create logger");
        logger.set_log_level(Level::Info);
        logger.start();

        let s = String::from("String");
        let i = 17;
        let ind = Level::Trace;
        info!(logger, "info from async: {} #{} of {}", s, i, ind);

        logger.stop();
    }

    #[test]
    fn test_logger_multi_threads() {
        let mut logger =
            AsyncLogger::new(BASENAME.into(), FILE_SIZE, 2).expect("failed to create logger");
        logger.set_log_level(Level::Info);
        logger.start();

        info!(logger, "main thread");

        let x = Arc::new(logger);
        let y = Arc::clone(&x);
        let t = std::thread::spawn(move || {
            warn!(y, "child thread #{:?} bang!", std::thread::current().id());
        });
        t.join().unwrap();

        match Arc::try_unwrap(x) {
            Ok(mut l) => l.stop(),
            Err(_) => eprintln!("failed to stop"),
        }
    }
}
