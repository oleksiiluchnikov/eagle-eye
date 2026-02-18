use super::super::output::{self, resolve_config};
use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};
use std::path::Path;

pub fn build() -> Command {
    Command::new("add-from-path")
        .about("Add an item from a local file path")
        .arg(
            Arg::new("path")
                .value_name("PATH")
                .help("Local file path")
                .required(true),
        )
        .arg(
            Arg::new("name")
                .value_name("NAME")
                .help("Display name for the item")
                .required(true),
        )
        .arg(
            Arg::new("website")
                .long("website")
                .value_name("URL")
                .help("Source website URL"),
        )
        .arg(
            Arg::new("annotation")
                .long("annotation")
                .value_name("TEXT")
                .help("Annotation text"),
        )
        .arg(
            Arg::new("tags")
                .long("tags")
                .value_name("TAGS")
                .help("Comma-separated tags"),
        )
        .arg(
            Arg::new("folder-id")
                .long("folder-id")
                .value_name("ID")
                .help("Target folder ID"),
        )
        .arg(
            Arg::new("if-exists")
                .long("if-exists")
                .value_name("ACTION")
                .help("Behavior when item already exists: skip or error (default: error)")
                .value_parser(["skip", "error"])
                .default_value("error"),
        )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = resolve_config(matches);
    let path_str = matches.get_one::<String>("path").expect("path is required");
    let name = matches.get_one::<String>("name").expect("name is required");
    let if_exists = matches
        .get_one::<String>("if-exists")
        .map(|s| s.as_str())
        .unwrap_or("error");

    if config.dry_run {
        eprintln!("dry-run: would add item from path {}", path_str);
        return Ok(());
    }

    let path = Path::new(path_str);
    let website = matches.get_one::<String>("website").map(|s| s.as_str());
    let annotation = matches.get_one::<String>("annotation").map(|s| s.as_str());
    let tags: Option<Vec<String>> = matches
        .get_one::<String>("tags")
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
    let folder_id = matches.get_one::<String>("folder-id").map(|s| s.as_str());

    match client
        .item()
        .add_from_path(path, name, website, annotation, tags.as_deref(), folder_id)
        .await
    {
        Ok(result) => {
            output::output(&result, &config)?;
        }
        Err(e) => {
            if if_exists == "skip" {
                if !config.quiet {
                    eprintln!("Skipped (--if-exists skip): {}", e);
                }
                return Ok(());
            }
            return Err(e);
        }
    }
    Ok(())
}
