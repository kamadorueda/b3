// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::ffi::OsString;

use clap::Arg;
use clap::Command;
use log::LevelFilter;

use super::cli_action::CliAction;
use super::cli_args::CliArgs;

pub(crate) fn parse<I, T>(args: I) -> CliArgs
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let parser = Command::new("Toros")
        .version("0.1.0")
        .about("Reproducible builds, dev envs and deployments.")
        .arg(
            Arg::new("log_level")
                .default_value("info")
                .help("Verbosity of the logger")
                .long("log-level")
                .possible_values(["trace", "debug", "info", "warn", "error"]),
        )
        .subcommand(
            Command::new("eval")
                .about("Parse and print a simplified version of the input.")
                .arg(
                    Arg::new("entrypoint")
                        .help("Top-level file of the project.")
                        .required(true),
                ),
        )
        .after_help(
            #[cfg_attr(rustfmt, rustfmt_skip)]
            "\
            ---\n\
            By:      Kevin Amado <https://patreon.com/kamadorueda>\n\
            Source:  https://github.com/kamadorueda/toros\n\
            License: GNU Affero General Public License v3.0 only\
            ",
        )
        .arg_required_else_help(true)
        .disable_help_subcommand(true)
        .term_width(80);

    let matches = parser.get_matches_from(args);

    let log_level = match matches.value_of("log_level") {
        Some("trace") => LevelFilter::Trace,
        Some("debug") => LevelFilter::Debug,
        Some("info") => LevelFilter::Info,
        Some("warn") => LevelFilter::Warn,
        Some("error") => LevelFilter::Error,
        _ => unreachable!(),
    };

    match matches.subcommand() {
        Some(("eval", matches)) => {
            let entrypoint =
                matches.value_of("entrypoint").unwrap().to_string();

            CliArgs { action: CliAction::Eval { entrypoint }, log_level }
        }
        _ => unreachable!(),
    }
}
