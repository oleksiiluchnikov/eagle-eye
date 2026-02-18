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
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = resolve_config(matches);
    let path_str = matches.get_one::<String>("path").expect("path is required");
    let name = matches.get_one::<String>("name").expect("name is required");

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

    let result = client
        .item()
        .add_from_path(path, name, website, annotation, tags.as_deref(), folder_id)
        .await?;
    output::output(&result, &config)?;
    Ok(())
}
