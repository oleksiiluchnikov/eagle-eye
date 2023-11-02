use clap::{Arg, ArgMatches, ArgAction, Command};
use crate::lib::client::EagleClient;
pub mod info;
pub mod list;
pub mod thumbnail;

pub fn build() -> Command {
                Command::new("item")
            .about("Item")
            // .arg(
            //     Arg::new("add")
            //     .help("Add item")
            //     // TODO: Add arguments
            //     ) 
            // .arg(
            //     Arg::new("info")
            //     .short('i')
            //     .long("info")
            //     .help("Show item info")
            //     )
            .subcommand(list::build())
            .subcommand(thumbnail::build())
            .subcommand(info::build())
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        Some(("info", info_matches)) => {
            info::execute(&client, info_matches).await?;
        },
        Some(("list", list_matches)) => {
            list::execute(&client, list_matches).await?;
        },
        Some(("thumbnail", thumbnail_matches)) => {
            thumbnail::execute(&client, thumbnail_matches).await?;
        },
        _ => {
            println!("No subcommand was used");
        }
    }
    Ok(())
}
