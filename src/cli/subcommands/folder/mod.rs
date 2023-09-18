pub mod list;
pub mod rename;
use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, ArgAction, Command};
use crate::lib::types::*;


pub fn build() -> Command {
    Command::new("folder")
        .about("Folder")

        .subcommand(
            Command::new("create")
            .about("Create folder")

            .arg(
                Arg::new("folder_name")
                .value_name("FOLDER_NAME")
                .help("Specify folder name")
                .required(true)
                // Type: String
                )

            .arg(
                Arg::new("parent_folder_id")
                .value_name("PARENT_FOLDER_ID")
                .help("Specify parent folder")
                .required(false)
                .default_value("")
                )
            )

            .subcommand(
                Command::new("rename")
                .about("Rename folder")
                .arg(
                    Arg::new("folder_id")
                    .value_name("FOLDER_ID")
                    .help("Specify folder id")
                    .required(true)
                    // Type: u64
                    )

                .arg(
                    Arg::new("new_name")
                    .value_name("NEW_NAME")
                    .help("Specify new name")
                    .required(true)
                    // Type: String
                    )
                )

            .subcommand(
                Command::new("update")
                .about("Update folder")

                .arg(
                    Arg::new("folder_id")
                    .value_name("FOLDER_ID")
                    .help("Specify folder id")
                    .required(true)
                    // Type: u64
                    )

                .arg(
                    Arg::new("new_name")
                    .value_name("NEW_NAME")
                    .help("Specify new name")
                    .required(false)
                    .default_value("")
                    // Type: String
                    )

                .arg(
                    Arg::new("new_description")
                    .value_name("NEW_DESCRIPTION")
                    .help("Specify new description")
                    .required(false)
                    .default_value("")
                    // Type: String
                    )

                .arg(
                    Arg::new("new_color")
                    .value_name("NEW_COLOR")
                    .help("Specify new color")
                    .required(false)
                    .default_value("")
                    // Type: String
                    )
                )

            .subcommand(
                Command::new("list")
                .about("List folders")

                .arg(
                    Arg::new("root_folder")
                    .short('r')
                    .long("root")
                    .help("Specify root folder")
                    .required(false)
                    .default_value("")
                    )

                .arg(
                    Arg::new("tree")
                    .short('t')
                    .long("tree")
                    .help("Show folder tree")
                    .action(ArgAction::SetTrue)
                    )

                .arg(
                    Arg::new("nesting_level")
                    .short('n')
                    .long("nesting-level")
                    .help("Specify nesting level")
                    .required(false)
                    .default_value("0")
                    )

                .arg(
                    Arg::new("recursive")
                    .short('R')
                    .long("recursive")
                    .help("Show folder tree recursively")
                    .action(ArgAction::SetTrue)
                    )
                )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
    ) -> Result<(), Box<dyn std::error::Error>> {

    match matches.subcommand() {
        Some(("list", matches)) => {
            list::execute(client, matches).await?;
        }
        Some(("create", matches)) => {
            todo!();
        }
        Some(("rename", matches)) => {
            todo!();
        }
        Some(("update", matches)) => {
            todo!();
        }
        _ => {}
    }


    Ok(())
    }
