// src/logger.rs
use log::{Level, Log, Metadata, Record};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: Level,
    pub message: String,
}

#[derive(Debug)]
pub struct MultiLogger {
    pub console: env_logger::Logger,
    pub buffer: Arc<Mutex<VecDeque<LogEntry>>>,
}

impl MultiLogger {
    pub fn new(buffer: Arc<Mutex<VecDeque<LogEntry>>>) -> Self {
        let console = env_logger::Builder::from_default_env().build();
        Self { console, buffer }
    }
}

impl Log for MultiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.console.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        self.console.log(record);

        if record.target().starts_with("coppercrab") {
            if let Ok(mut buf) = self.buffer.lock() {
                buf.push_back(LogEntry {
                    level: record.level(),
                    message: format!("[{}] {}", record.target(), record.args()),
                });
                if buf.len() > 100 {
                    buf.pop_front();
                }
            }
        }
    }

    fn flush(&self) {
        self.console.flush();
    }
}
