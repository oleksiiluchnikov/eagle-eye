use super::super::output::{self, resolve_format};
use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("update")
        .about("Update an item's properties")
        .arg(
            Arg::new("id")
                .value_name("ID")
                .help("Item ID to update")
                .required(true),
        )
        .arg(
            Arg::new("tags")
                .long("tags")
                .value_name("TAGS")
                .help("Comma-separated tags (replaces existing tags)"),
        )
        .arg(
            Arg::new("annotation")
                .long("annotation")
                .value_name("TEXT")
                .help("Annotation text"),
        )
        .arg(
            Arg::new("url")
                .long("url")
                .value_name("URL")
                .help("Source URL"),
        )
        .arg(
            Arg::new("star")
                .long("star")
                .value_name("RATING")
                .help("Star rating (0-5)")
                .value_parser(clap::value_parser!(u8)),
        )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let fmt = resolve_format(matches);
    let id = matches.get_one::<String>("id").expect("id is required");

    let tags: Option<Vec<String>> = matches
        .get_one::<String>("tags")
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
    let annotation = matches.get_one::<String>("annotation").map(|s| s.as_str());
    let url = matches.get_one::<String>("url").map(|s| s.as_str());
    let star = matches.get_one::<u8>("star").copied();

    let data = client
        .item()
        .update(id, tags.as_deref(), annotation, url, star)
        .await?
        .data;
    output::output(&data, &fmt)?;
    Ok(())
}
