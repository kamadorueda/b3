// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only



use anyhow::Context;

pub fn main<Args>(args: Args) -> anyhow::Result<()>
where
    Args: IntoIterator<Item = String>,
{
    let args = clap::Command::new("b3")
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
            clap::Command::new("simplify")
                .about("Simplify the provided expression as much as possible.")
                .arg(
                    clap::Arg::new("path")
                        .help("Path to evaluate")
                        .value_parser(clap::value_parser!(std::path::PathBuf)),
                ),
        )
        .subcommand(
            clap::Command::new("parse")
                .about("Parse the provided expression and pretty print it.")
                .arg(
                    clap::Arg::new("path")
                        .help("Path to evaluate")
                        .value_parser(clap::value_parser!(std::path::PathBuf))
                        .required(true),
                ),
        )
        .arg_required_else_help(true)
        .term_width(80)
        .try_get_matches_from(args)?;

    log::set_max_level(
        args.get_one::<String>("log-level")
            .expect("Unable to parse log-level as a String.")
            .as_str()
            .parse::<log::LevelFilter>()
            .expect("Unable to parse log-level as a log::LevelFilter"),
    );
    log::debug!("{args:#?}");

    match args.subcommand() {
        Some(("parse", args)) => parse(args),
        Some(("simplify", args)) => simplify(args),
        Some((subcommand, _)) => {
            Err(anyhow::anyhow!("Unknown subcommand: {subcommand}"))
        },
        _ => Ok(()),
    }
}

fn parse(args: &clap::ArgMatches) -> anyhow::Result<()> {
    let path = args
        .try_get_one::<std::path::PathBuf>("path")?
        .ok_or_else(|| anyhow::anyhow!("Unable to parse path"))?;

    let input = std::fs::read_to_string(path)
        .with_context(|| format!("Unable to read file: {path:?}"))?;

    let expression = crate::core::parse::parse(input);

    log::info!(target: "stdout", "{expression:#?}");

    Ok(())
}

fn simplify(args: &clap::ArgMatches) -> anyhow::Result<()> {
    let path = args
        .try_get_one::<std::path::PathBuf>("path")?
        .ok_or_else(|| anyhow::anyhow!("Unable to parse path"))?;

    let expression = crate::core::simplify::simplify(path)?;

    log::info!(target: "stdout", "{expression:#?}");

    Ok(())
}
