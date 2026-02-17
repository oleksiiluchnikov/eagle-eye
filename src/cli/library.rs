use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};
use std::path::Path;

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        Some(("info", info_matches)) => {
            let data = client.library().info().await?.data;
            if info_matches.get_flag("folders") {
                println!("{}", serde_json::to_string_pretty(&data.folders)?);
            } else if info_matches.get_flag("smart_folders") {
                println!("{}", serde_json::to_string_pretty(&data.smart_folders)?);
            } else if info_matches.get_flag("quick_access") {
                println!("{}", serde_json::to_string_pretty(&data.quick_access)?);
            } else if info_matches.get_flag("tags_groups") {
                println!("{}", serde_json::to_string_pretty(&data.tags_groups)?);
            } else if info_matches.get_flag("modification_time") {
                println!("{}", data.modification_time);
            } else {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        }
        Some(("history", _)) => {
            let result = client.library().history().await?;
            println!("{}", serde_json::to_string_pretty(&result.data)?);
        }
        Some(("switch", switch_matches)) => {
            let path = switch_matches
                .get_one::<String>("path")
                .expect("path is required");
            let result = client.library().switch(Path::new(path)).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Some(("library", library_matches)) => {
            let data = client.library().info().await?.data;
            if library_matches.get_flag("path") {
                println!("{}", data.library.path);
            } else if library_matches.get_flag("name") {
                println!("{}", data.library.name);
            } else {
                println!("{}", serde_json::to_string_pretty(&data.library)?);
            }
        }
        _ => {}
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
                        .num_args(0),
                )
                .arg(
                    Arg::new("smart_folders")
                        .short('s')
                        .long("smart-folders")
                        .help("Show smart folders")
                        .action(clap::ArgAction::SetTrue)
                        .num_args(0),
                )
                .arg(
                    Arg::new("quick_access")
                        .short('q')
                        .long("quick-access")
                        .help("Show quick access")
                        .action(clap::ArgAction::SetTrue)
                        .num_args(0),
                )
                .arg(
                    Arg::new("tags_groups")
                        .short('t')
                        .long("tags-groups")
                        .help("Show tags groups")
                        .action(clap::ArgAction::SetTrue)
                        .num_args(0),
                )
                .arg(
                    Arg::new("modification_time")
                        .short('m')
                        .long("modification-time")
                        .help("Show modification time")
                        .action(clap::ArgAction::SetTrue)
                        .num_args(0),
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
            Command::new("library")
                .about("Library")
                .arg(
                    Arg::new("path")
                        .short('p')
                        .long("path")
                        .help("Current working library path")
                        .required(false)
                        .num_args(0),
                )
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .help("Current working library name")
                        .required(false)
                        .num_args(0),
                ),
        )
}
