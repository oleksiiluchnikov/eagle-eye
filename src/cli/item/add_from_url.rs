use super::super::output::{self, resolve_config};
use crate::lib::client::EagleClient;
use crate::lib::types::{AddFromUrlParams, OutgoingHttpHeaders};
use clap::{Arg, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("add-from-url")
        .about("Add an item from a URL")
        .arg(
            Arg::new("url")
                .value_name("URL")
                .help("URL to download from")
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
            Arg::new("tags")
                .long("tags")
                .value_name("TAGS")
                .help("Comma-separated tags"),
        )
        .arg(
            Arg::new("annotation")
                .long("annotation")
                .value_name("TEXT")
                .help("Annotation text"),
        )
        .arg(
            Arg::new("folder-id")
                .long("folder-id")
                .value_name("ID")
                .help("Target folder ID"),
        )
        .arg(
            Arg::new("star")
                .long("star")
                .value_name("RATING")
                .help("Star rating (0-5)")
                .value_parser(clap::value_parser!(u8)),
        )
        .arg(
            Arg::new("modification-time")
                .long("modification-time")
                .value_name("TIMESTAMP")
                .help("Modification time (Unix timestamp in milliseconds)")
                .value_parser(clap::value_parser!(u64)),
        )
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .value_name("KEY:VALUE")
                .help("Custom HTTP header for downloading (can be repeated)")
                .action(clap::ArgAction::Append),
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
    let url = matches.get_one::<String>("url").expect("url is required");
    let name = matches.get_one::<String>("name").expect("name is required");
    let if_exists = matches
        .get_one::<String>("if-exists")
        .map(|s| s.as_str())
        .unwrap_or("error");

    if config.dry_run {
        eprintln!("dry-run: would add item from URL {}", url);
        return Ok(());
    }

    let website = matches.get_one::<String>("website").cloned();
    let tags: Option<Vec<String>> = matches
        .get_one::<String>("tags")
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
    let annotation = matches.get_one::<String>("annotation").cloned();
    let folder_id = matches.get_one::<String>("folder-id").cloned();
    let star = matches.get_one::<u8>("star").copied();
    let modification_time = matches.get_one::<u64>("modification-time").copied();
    let headers: Option<OutgoingHttpHeaders> = matches.get_many::<String>("header").map(|vals| {
        vals.filter_map(|h| {
            let (key, value) = h.split_once(':')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
    });

    let params = AddFromUrlParams {
        url: url.to_string(),
        name: name.to_string(),
        website,
        tags,
        annotation,
        folder_id,
        star,
        modification_time,
        headers,
    };

    match client.item().add_from_url(&params).await {
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
