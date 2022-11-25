// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;

pub struct Logger<Buffer>
where
    Buffer: std::io::Write,
{
    started_at: std::time::Instant,
    buffer: Arc<Mutex<Buffer>>,
    buffer_stdout: bool,
}

pub fn initialize() -> anyhow::Result<()> {
    let logger = Box::new(Logger {
        started_at: std::time::Instant::now(),
        buffer: Arc::new(Mutex::new(std::io::stderr())),
        buffer_stdout: false,
    });

    log::set_logger(Box::leak(logger))
        .map_err(|error| anyhow::anyhow!(error))
        .context("Initialize global logger implementation")
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
        let mut buffer = self.buffer.lock().unwrap_or_else(|error| {
            panic!("Unable to acquire the logger's lock: {error}.");
        });

        let metadata = record.metadata();

        let target = metadata.target();
        let args = record.args();

        if target == "stdout" {
            if self.buffer_stdout {
                writeln!(buffer, "{args}")
            } else {
                writeln!(std::io::stdout().lock(), "{args}")
            }
        } else if target == "stderr" {
            if self.buffer_stdout {
                writeln!(buffer, "{args}")
            } else {
                writeln!(std::io::stderr().lock(), "{args}")
            }
        } else {
            let level = metadata.level();

            match level {
                log::Level::Trace => {
                    let elapsed = self.started_at.elapsed().as_secs_f64();
                    let line = record.line().unwrap_or(u32::MAX);

                    writeln!(
                        buffer,
                        "{level} @ {elapsed:>8.3}, {target}#{line}: {args}",
                    )
                },
                log::Level::Debug => {
                    let line = record.line().unwrap_or(u32::MAX);

                    writeln!(buffer, "{level}, {target}#{line}: {args}")
                },
                _ => {
                    writeln!(buffer, "{level}: {args}")
                },
            }
        }
        .unwrap_or_else(|error| {
            panic!("Unable to write to the logger's buffer: {error}.")
        })
    }

    fn flush(&self) {}
}

pub fn bytes() -> Box<Logger<std::io::Cursor<Vec<u8>>>> {
    Box::new(Logger {
        started_at: std::time::Instant::now(),
        buffer: Arc::new(Mutex::new(std::io::Cursor::new(Vec::new()))),
        buffer_stdout: true,
    })
}

impl Logger<std::io::Cursor<Vec<u8>>> {
    pub fn get_buffer(&self) -> Arc<Mutex<std::io::Cursor<Vec<u8>>>> {
        self.buffer.clone()
    }
}
