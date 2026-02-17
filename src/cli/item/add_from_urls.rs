use crate::lib::client::EagleClient;
use crate::lib::types::Item;
use clap::{Arg, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("add-from-urls")
        .about("Add multiple items from URLs (JSON input)")
        .arg(
            Arg::new("json")
                .value_name("JSON")
                .help(
                    "JSON array of items, each with \"url\" and optional \"name\", \"tags\", etc.",
                )
                .required(true),
        )
        .arg(
            Arg::new("folder-id")
                .long("folder-id")
                .value_name("ID")
                .help("Target folder ID for all items"),
        )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_str = match matches.get_one::<String>("json") {
        Some(j) => j,
        None => {
            println!("No JSON input provided");
            return Ok(());
        }
    };

    let items: Vec<Item> = serde_json::from_str(json_str)?;
    let folder_id = matches.get_one::<String>("folder-id").map(|s| s.as_str());

    let result = client.item().add_from_urls(&items, folder_id).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
