// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::cell::RefCell;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use log::Level;
use log::Metadata;
use log::Record;

pub struct Logger<Output>
where
    Output: Write,
{
    started_at: Instant,
    output:     Arc<Mutex<RefCell<Output>>>,
}

impl<Output> Logger<Output>
where
    Output: Write,
{
    pub fn new(output: Arc<Mutex<RefCell<Output>>>) -> Logger<Output> {
        Logger { started_at: Instant::now(), output }
    }
}

impl<Output> log::Log for Logger<Output>
where
    Output: Send + Sync + Write,
{
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let metadata = record.metadata();

        if let Ok(output) = self.output.lock() {
            let level = metadata.level();

            match level {
                Level::Trace => {
                    writeln!(
                        output.borrow_mut(),
                        "[{} @ {:>8.3} @ {}#{}]: {}",
                        level,
                        self.started_at.elapsed().as_secs_f64(),
                        metadata.target(),
                        record.line().unwrap_or(u32::MAX),
                        record.args(),
                    )
                    .unwrap();
                }
                Level::Debug => {
                    writeln!(
                        output.borrow_mut(),
                        "[{} @ {}#{}]: {}",
                        level,
                        metadata.target(),
                        record.line().unwrap_or(u32::MAX),
                        record.args(),
                    )
                    .unwrap();
                }
                _ => {
                    writeln!(
                        output.borrow_mut(),
                        "[{}]: {}",
                        level,
                        record.args()
                    )
                    .unwrap();
                }
            }
        }
    }

    fn flush(&self) {}
}
