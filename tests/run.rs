// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;

#[test]
fn bin() -> anyhow::Result<()> {
    let should_update = std::env::var("UPDATE").is_ok();

    let logger = b3::logger::bytes();

    let buffer = logger.get_buffer();

    log::set_logger(Box::leak(logger)).unwrap_or_else(|error| {
        panic!("Unable to initialize logger: {error}");
    });

    for path in find_files("tests")
        .into_iter()
        .filter(|path| path.ends_with("/args.lst"))
    {
        let args: Vec<String> = ["b3"]
            .into_iter()
            .chain(
                std::fs::read_to_string(&path)
                    .with_context(|| format!("Unable to read file: {path:?}"))?
                    .lines(),
            )
            .map(String::from)
            .collect();

        let output = b3::cli::main(args).map_or_else(
            |error| anyhow::Ok(error.to_string().into_bytes()),
            |_| {
                let mut output_log = Vec::new();
                let mut cursor = buffer
                    .lock()
                    .map_err(|error| anyhow::anyhow!("{error}"))?;
                cursor.seek(SeekFrom::Start(0))?;
                cursor.read_to_end(&mut output_log)?;
                cursor.get_mut().clear();
                Ok(output_log)
            },
        )?;

        let output_path = path.with_file_name("output.log");

        if should_update {
            std::fs::File::create(&output_path)?.write_all(&output)?;
        }

        assert_eq!(
            String::from_utf8(output)?,
            std::fs::read_to_string(&output_path)?
        );
    }

    Ok(())
}

fn find_files(path: &str) -> Vec<PathBuf> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_owned())
        .collect()
}
