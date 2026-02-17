use crate::lib::client::EagleClient;
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
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = match matches.get_one::<String>("url") {
        Some(u) => u,
        None => {
            println!("No URL provided");
            return Ok(());
        }
    };
    let name = match matches.get_one::<String>("name") {
        Some(n) => n,
        None => {
            println!("No name provided");
            return Ok(());
        }
    };

    let website = matches.get_one::<String>("website").map(|s| s.as_str());
    let tags: Option<Vec<String>> = matches
        .get_one::<String>("tags")
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
    let annotation = matches.get_one::<String>("annotation").map(|s| s.as_str());
    let folder_id = matches.get_one::<String>("folder-id").map(|s| s.as_str());

    let result = client
        .item()
        .add_from_url(url, name, website, tags.as_deref(), annotation, folder_id)
        .await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
