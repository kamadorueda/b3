// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::cell::RefCell;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

use toros::logger::Logger;

#[test]
fn test_bin_toros() {
    let should_update = std::env::var("UPDATE").is_ok();

    let output = Arc::new(Mutex::new(RefCell::new(Cursor::new(Vec::new()))));

    let logger = Logger::new(output.clone());
    let logger = Box::new(logger);
    log::set_boxed_logger(logger).unwrap();

    for path in find_files("./tests") {
        if path.ends_with("/cli.args") {
            let path_content = std::fs::read_to_string(&path).unwrap();
            let path_output_log = path.replace("/cli.args", "/output.log");

            let mut cli_args = vec!["toros"];
            for cli_arg in path_content.lines() {
                cli_args.push(cli_arg);
            }

            toros::cli::bin_toros::main(&cli_args);

            let mut output_log = Vec::new();
            let output = output.lock().unwrap();
            {
                let mut cursor = output.borrow_mut();
                cursor.seek(SeekFrom::Start(0)).unwrap();
                cursor.read_to_end(&mut output_log).unwrap();
            };
            output.replace(Cursor::new(Vec::new()));

            if should_update {
                std::fs::File::create(&path_output_log)
                    .unwrap()
                    .write_all(&output_log)
                    .unwrap();
            }

            assert_eq!(
                String::from_utf8(output_log).unwrap(),
                std::fs::read_to_string(&path_output_log).unwrap()
            );
        }
    }
}

fn find_files(path: &str) -> Vec<String> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            Err(_) => None,
        })
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_str().unwrap().to_string())
        .collect()
}
