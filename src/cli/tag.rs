use super::output::{self, resolve_config};
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
    let config = resolve_config(matches);

    match matches.subcommand() {
        Some(("list", _)) => {
            let result = client.tag().list().await?;
            output::output(&result.data, &config)?;
        }
        Some(("all", _)) => {
            let result = client.tag().all().await?;
            output::output(&result.data, &config)?;
        }
        Some(("list-recent", _)) => {
            let result = client.tag().list_recent().await?;
            output::output(&result.data, &config)?;
        }
        Some(("groups", _)) => {
            let result = client.tag().groups().await?;
            output::output(&result.data, &config)?;
        }
        _ => {
            eprintln!("Error: No subcommand was used. Try: list, all, list-recent, groups");
            std::process::exit(super::output::exit_code::USAGE);
        }
    }

    Ok(())
}
