// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

pub fn main<Args>(args: Args) -> Result<(), clap::Error>
where
    Args: IntoIterator<Item = String>,
{
    let parsed = clap::Command::new("b3")
        .about("Bit by bit reproducible builds, dev envs and deployments.")
        .arg(
            clap::Arg::new("log-level")
                .help("Verbosity level")
                .long("log-level")
                .default_value("info")
                .value_parser([
                    "trace", "debug", "info", "warn", "error", "off",
                ])
                .required(false),
        )
        .subcommand(
            clap::Command::new("eval")
                .about("Evaluate the provided expression as much as possible.")
                .arg(
                    clap::Arg::new("path")
                        .help("Path to evaluate")
                        .value_parser(clap::value_parser!(std::path::PathBuf)),
                ),
        )
        .arg_required_else_help(true)
        .term_width(80)
        .try_get_matches_from(args)?;

    log::set_max_level(
        parsed
            .get_one::<String>("log-level")
            .expect("Unable to parse log-level as a String.")
            .as_str()
            .parse::<log::LevelFilter>()
            .expect("Unable to parse log-level as a log::LevelFilter"),
    );
    log::debug!("{parsed:#?}");

    match parsed.subcommand() {
        Some(("eval", parsed)) => eval(parsed),
        _ => Ok(()),
    }
}

fn eval(_: &clap::ArgMatches) -> Result<(), clap::Error> {
    Ok(())
}
