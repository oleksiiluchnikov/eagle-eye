use super::output::{self, output_plain, resolve_config};
use crate::lib::client::EagleClient;
use crate::lib::types::GetLibraryIconParams;
use clap::{Arg, ArgMatches, Command};
use std::io::{self, Write};
use std::path::Path;

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = resolve_config(matches);

    match matches.subcommand() {
        Some(("info", info_matches)) => {
            let data = client.library().info().await?.data;
            if info_matches.get_flag("folders") {
                output::output(&data.folders, &config)?;
            } else if info_matches.get_flag("smart_folders") {
                output::output(&data.smart_folders, &config)?;
            } else if info_matches.get_flag("quick_access") {
                output::output(&data.quick_access, &config)?;
            } else if info_matches.get_flag("tags_groups") {
                output::output(&data.tags_groups, &config)?;
            } else if info_matches.get_flag("modification_time") {
                output_plain(&data.modification_time.to_string());
            } else {
                output::output(&data, &config)?;
            }
        }
        Some(("history", _)) => {
            let result = client.library().history().await?;
            output::output(&result.data, &config)?;
        }
        Some(("switch", switch_matches)) => {
            let config = resolve_config(switch_matches);
            let path = switch_matches
                .get_one::<String>("path")
                .expect("path is required");
            if config.dry_run {
                eprintln!("dry-run: would switch to library {}", path);
                return Ok(());
            }
            let result = client.library().switch(Path::new(path)).await?;
            // switch returns only {status}, output it directly
            output::output(&result, &config)?;
        }
        Some(("current", current_matches)) => {
            let data = client.library().info().await?.data;
            if current_matches.get_flag("path") {
                output_plain(&data.library.path);
            } else if current_matches.get_flag("name") {
                output_plain(&data.library.name);
            } else {
                output::output(&data.library, &config)?;
            }
        }
        Some(("icon", icon_matches)) => {
            let library_path = if let Some(p) = icon_matches.get_one::<String>("library-path") {
                p.clone()
            } else {
                // Default: use current library path
                let data = client.library().info().await?.data;
                data.library.path.clone()
            };
            let params = GetLibraryIconParams { library_path };
            let bytes = client.library().icon(params).await?;
            let mut out = io::stdout().lock();
            out.write_all(&bytes)?;
            out.flush()?;
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
        .subcommand(
            Command::new("icon")
                .about("Get library icon (writes raw image bytes to stdout)")
                .arg(
                    Arg::new("library-path")
                        .short('l')
                        .long("library-path")
                        .value_name("PATH")
                        .help("Library path (defaults to current library)"),
                ),
        )
}
