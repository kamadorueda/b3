// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::cell::RefCell;
use std::io::stdout;
use std::sync::Arc;
use std::sync::Mutex;

use toros::logger::Logger;

fn main() {
    let cli_args = std::env::args();

    let output = Arc::new(Mutex::new(RefCell::new(stdout())));

    let logger = Logger::new(output);
    let logger = Box::new(logger);
    log::set_boxed_logger(logger).unwrap();

    let exit_code = toros::cli::bin_toros::main(cli_args);

    std::process::exit(exit_code);
}
