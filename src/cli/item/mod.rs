use crate::lib::client::EagleClient;
use clap::{ArgMatches, Command};
pub mod add_bookmark;
pub mod add_from_path;
pub mod add_from_paths;
pub mod add_from_url;
pub mod add_from_urls;
pub mod info;
pub mod list;
pub mod refresh_palette;
pub mod refresh_thumbnail;
pub mod thumbnail;
pub mod update;

pub fn build() -> Command {
    Command::new("item")
        .about("Item")
        .subcommand(list::build())
        .subcommand(info::build())
        .subcommand(thumbnail::build())
        .subcommand(update::build())
        .subcommand(add_from_url::build())
        .subcommand(add_from_urls::build())
        .subcommand(add_from_path::build())
        .subcommand(add_from_paths::build())
        .subcommand(add_bookmark::build())
        .subcommand(refresh_palette::build())
        .subcommand(refresh_thumbnail::build())
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        Some(("info", sub_matches)) => info::execute(client, sub_matches).await?,
        Some(("list", sub_matches)) => list::execute(client, sub_matches).await?,
        Some(("thumbnail", sub_matches)) => thumbnail::execute(client, sub_matches).await?,
        Some(("update", sub_matches)) => update::execute(client, sub_matches).await?,
        Some(("add-from-url", sub_matches)) => add_from_url::execute(client, sub_matches).await?,
        Some(("add-from-urls", sub_matches)) => add_from_urls::execute(client, sub_matches).await?,
        Some(("add-from-path", sub_matches)) => add_from_path::execute(client, sub_matches).await?,
        Some(("add-from-paths", sub_matches)) => {
            add_from_paths::execute(client, sub_matches).await?
        }
        Some(("add-bookmark", sub_matches)) => add_bookmark::execute(client, sub_matches).await?,
        Some(("refresh-palette", sub_matches)) => {
            refresh_palette::execute(client, sub_matches).await?
        }
        Some(("refresh-thumbnail", sub_matches)) => {
            refresh_thumbnail::execute(client, sub_matches).await?
        }
        _ => {
            println!("No subcommand was used");
        }
    }
    Ok(())
}
