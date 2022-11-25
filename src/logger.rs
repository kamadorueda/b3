// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Logger<Buffer>
where
    Buffer: std::io::Write,
{
    started_at: std::time::Instant,
    buffer: Arc<Mutex<RefCell<Buffer>>>,
}

pub fn initialize() {
    let logger = Box::new(Logger {
        started_at: std::time::Instant::now(),
        buffer: Arc::new(Mutex::new(RefCell::new(std::io::stderr()))),
    });

    log::set_logger(Box::leak(logger)).unwrap_or_else(|error| {
        panic!("Unable to initialize logger: {error}");
    });
}

impl<Buffer> log::Log for Logger<Buffer>
where
    Buffer: Send + Sync + std::io::Write,
{
    #[inline(always)]
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let buffer_cell = self.buffer.lock().unwrap_or_else(|error| {
            panic!("Unable to acquire the logger's lock: {error}.");
        });

        let mut buffer = buffer_cell.try_borrow_mut().unwrap_or_else(|error| {
            panic!("Unable to borrow the logger's buffer: {error}.")
        });

        let metadata = record.metadata();
        let level = metadata.level();
        let args = record.args();

        match level {
            log::Level::Trace => {
                let elapsed = self.started_at.elapsed().as_secs_f64();
                let target = metadata.target();
                let line = record.line().unwrap_or(u32::MAX);

                writeln!(
                    buffer,
                    "{level} @ {elapsed:>8.3}, {target}#{line}: {args}",
                )
            },
            log::Level::Debug => {
                let target = metadata.target();
                let line = record.line().unwrap_or(u32::MAX);

                writeln!(buffer, "{level}, {target}#{line}: {args}",)
            },
            _ => {
                writeln!(buffer, "{level}: {args}")
            },
        }
        .unwrap_or_else(|error| {
            panic!("Unable to write to the logger's buffer: {error}.")
        });
    }

    fn flush(&self) {}
}

pub fn bytes() -> Box<Logger<std::io::Cursor<Vec<u8>>>> {
    Box::new(Logger {
        started_at: std::time::Instant::now(),
        buffer: Arc::new(Mutex::new(RefCell::new(std::io::Cursor::new(
            Vec::new(),
        )))),
    })
}

impl Logger<std::io::Cursor<Vec<u8>>> {
    pub fn get_buffer(&self) -> Arc<Mutex<RefCell<std::io::Cursor<Vec<u8>>>>> {
        return self.buffer.clone();
    }
}
