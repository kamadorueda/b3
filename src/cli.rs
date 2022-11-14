pub fn main<Args>(args: Args) -> i32
where
    Args: IntoIterator<Item = String>,
{
    let parser = clap::Command::new("b3")
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
        .term_width(80);

    let parsed = parser.get_matches_from(args);

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
        _ => {
            // log::error!("Not subcommand was specified, please run with
            // --help")
            0
        },
    }
}

fn eval(parsed: &clap::ArgMatches) -> i32 {
    0
}
