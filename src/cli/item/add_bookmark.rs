use super::super::output::{self, resolve_config};
use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("add-bookmark")
        .about("Add a bookmark")
        .arg(
            Arg::new("url")
                .value_name("URL")
                .help("Bookmark URL")
                .required(true),
        )
        .arg(
            Arg::new("name")
                .value_name("NAME")
                .help("Display name for the bookmark")
                .required(true),
        )
        .arg(
            Arg::new("base64")
                .long("base64")
                .value_name("DATA")
                .help("Base64-encoded thumbnail image"),
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
            Arg::new("modification-time")
                .long("modification-time")
                .value_name("TIMESTAMP")
                .help("Modification time (Unix timestamp in milliseconds)")
                .value_parser(clap::value_parser!(u64)),
        )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = resolve_config(matches);
    let url = matches.get_one::<String>("url").expect("url is required");
    let name = matches.get_one::<String>("name").expect("name is required");

    if config.dry_run {
        eprintln!("dry-run: would add bookmark {}", url);
        return Ok(());
    }

    let base64 = matches.get_one::<String>("base64").map(|s| s.as_str());
    let tags: Option<Vec<String>> = matches
        .get_one::<String>("tags")
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
    let folder_id = matches.get_one::<String>("folder-id").map(|s| s.as_str());
    let modification_time = matches.get_one::<u64>("modification-time").copied();

    let result = client
        .item()
        .add_bookmark(
            url,
            name,
            base64,
            tags.as_deref(),
            folder_id,
            modification_time,
        )
        .await?;
    output::output(&result, &config)?;
    Ok(())
}
