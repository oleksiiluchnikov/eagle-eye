use crate::lib;
use clap::{ArgMatches, Command};

pub mod app;
pub mod folder;
pub mod item;
pub mod library;
pub mod tag;

pub const PORT: u16 = 41595;
const HOST: &str = "localhost";

pub fn get_matches() -> ArgMatches {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg_required_else_help(true)
        .subcommand(app::build())
        .subcommand(folder::build())
        .subcommand(item::build())
        .subcommand(library::build())
        .subcommand(tag::build())
        .get_matches()
}

pub async fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let matches = get_matches();
    let eagle_client = lib::client::EagleClient::new(HOST, PORT);

    match matches.subcommand() {
        Some(("app", app_matches)) => app::execute(&eagle_client, app_matches).await,
        Some(("folder", folder_matches)) => folder::execute(&eagle_client, folder_matches).await,
        Some(("item", item_matches)) => item::execute(&eagle_client, item_matches).await,
        Some(("library", library_matches)) => {
            library::execute(&eagle_client, library_matches).await
        }
        Some(("tag", tag_matches)) => tag::execute(&eagle_client, tag_matches).await,
        _ => {
            println!("No subcommand was used");
            Ok(())
        }
    }
}
