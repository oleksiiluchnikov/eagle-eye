use crate::lib::client::EagleClient;
use clap::{ArgMatches, Command};

pub fn build() -> Command {
    Command::new("tag")
        .about("Tag")
        .subcommand(Command::new("list").about("List all tags"))
        .subcommand(Command::new("all").about("Get all tag data (tags, recent, groups, starred)"))
        .subcommand(Command::new("list-recent").about("List recently used tags"))
        .subcommand(Command::new("groups").about("List tag groups"))
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        Some(("list", _)) => {
            let result = client.tag().list().await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Some(("all", _)) => {
            let result = client.tag().all().await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Some(("list-recent", _)) => {
            let result = client.tag().list_recent().await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Some(("groups", _)) => {
            let result = client.tag().groups().await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        _ => {
            println!("No subcommand was used. Try: list, all, list-recent, groups");
        }
    }

    Ok(())
}
