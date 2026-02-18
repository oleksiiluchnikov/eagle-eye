use super::output::{self, output_plain, resolve_format};
use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};
use std::path::Path;

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let fmt = resolve_format(matches);

    match matches.subcommand() {
        Some(("info", info_matches)) => {
            let data = client.library().info().await?.data;
            if info_matches.get_flag("folders") {
                output::output(&data.folders, &fmt)?;
            } else if info_matches.get_flag("smart_folders") {
                output::output(&data.smart_folders, &fmt)?;
            } else if info_matches.get_flag("quick_access") {
                output::output(&data.quick_access, &fmt)?;
            } else if info_matches.get_flag("tags_groups") {
                output::output(&data.tags_groups, &fmt)?;
            } else if info_matches.get_flag("modification_time") {
                output_plain(&data.modification_time.to_string());
            } else {
                output::output(&data, &fmt)?;
            }
        }
        Some(("history", _)) => {
            let result = client.library().history().await?;
            output::output(&result.data, &fmt)?;
        }
        Some(("switch", switch_matches)) => {
            let path = switch_matches
                .get_one::<String>("path")
                .expect("path is required");
            let result = client.library().switch(Path::new(path)).await?;
            // switch returns only {status}, output it directly
            output::output(&result, &fmt)?;
        }
        Some(("current", current_matches)) => {
            let data = client.library().info().await?.data;
            if current_matches.get_flag("path") {
                output_plain(&data.library.path);
            } else if current_matches.get_flag("name") {
                output_plain(&data.library.name);
            } else {
                output::output(&data.library, &fmt)?;
            }
        }
        _ => {
            eprintln!("Error: No subcommand was used. Try: info, history, switch, current");
            std::process::exit(super::output::exit_code::USAGE);
        }
    }
    Ok(())
}

pub fn build() -> Command {
    Command::new("library")
        .about("Library")
        .subcommand(
            Command::new("info")
                .about("Library info")
                .arg(
                    Arg::new("folders")
                        .short('f')
                        .long("folders")
                        .help("Show folders")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("smart_folders")
                        .short('s')
                        .long("smart-folders")
                        .help("Show smart folders")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("quick_access")
                        .short('q')
                        .long("quick-access")
                        .help("Show quick access")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("tags_groups")
                        .short('t')
                        .long("tags-groups")
                        .help("Show tags groups")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("modification_time")
                        .short('m')
                        .long("modification-time")
                        .help("Show modification time")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(Command::new("history").about("Library history"))
        .subcommand(
            Command::new("switch").about("Switch library").arg(
                Arg::new("path")
                    .short('p')
                    .long("path")
                    .help("Library path")
                    .required(true)
                    .num_args(1),
            ),
        )
        .subcommand(
            Command::new("current")
                .about("Current working library")
                .arg(
                    Arg::new("path")
                        .short('p')
                        .long("path")
                        .help("Current working library path")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .help("Current working library name")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
}
