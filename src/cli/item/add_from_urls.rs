use super::super::output::{self, resolve_config};
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
    let config = resolve_config(matches);
    let json_str = matches.get_one::<String>("json").expect("json is required");

    let items: Vec<Item> = serde_json::from_str(json_str)?;
    let folder_id = matches.get_one::<String>("folder-id").map(|s| s.as_str());

    if config.dry_run {
        eprintln!("dry-run: would add {} item(s) from URLs", items.len());
        return Ok(());
    }

    let result = client.item().add_from_urls(&items, folder_id).await?;
    output::output(&result, &config)?;
    Ok(())
}
