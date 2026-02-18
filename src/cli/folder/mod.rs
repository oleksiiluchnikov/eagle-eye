pub mod list;
pub mod rename;
use super::output::{self, resolve_config};
use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("folder")
        .about("Folder")
        .subcommand(Command::new("list-recent").about("List recently used folders"))
        .subcommand(
            Command::new("create")
                .about("Create folder")
                .arg(
                    Arg::new("folder_name")
                        .value_name("FOLDER_NAME")
                        .help("Specify folder name")
                        .required(true), // Type: String
                )
                .arg(
                    Arg::new("parent_folder_id")
                        .value_name("PARENT_FOLDER_ID")
                        .help("Specify parent folder")
                        .required(false)
                        .default_value(""),
                ),
        )
        .subcommand(
            Command::new("rename")
                .about("Rename folder")
                .arg(
                    Arg::new("folder_id")
                        .value_name("FOLDER_ID")
                        .help("Specify folder id")
                        .required(true), // Type: u64
                )
                .arg(
                    Arg::new("new_name")
                        .value_name("NEW_NAME")
                        .help("Specify new name")
                        .required(true), // Type: String
                ),
        )
        .subcommand(
            Command::new("update")
                .about("Update folder")
                .arg(
                    Arg::new("folder_id")
                        .value_name("FOLDER_ID")
                        .help("Specify folder id")
                        .required(true), // Type: u64
                )
                .arg(
                    Arg::new("new_name")
                        .value_name("NEW_NAME")
                        .help("Specify new name")
                        .required(false)
                        .default_value(""), // Type: String
                )
                .arg(
                    Arg::new("new_description")
                        .value_name("NEW_DESCRIPTION")
                        .help("Specify new description")
                        .required(false)
                        .default_value(""), // Type: String
                )
                .arg(
                    Arg::new("new_color")
                        .value_name("NEW_COLOR")
                        .help("Specify new color")
                        .required(false)
                        .default_value(""), // Type: String
                ),
        )
        .subcommand(list::build())
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        Some(("list", matches)) => {
            list::execute(client, matches).await?;
        }
        Some(("list-recent", sub_matches)) => {
            let config = resolve_config(sub_matches);
            let data = client.folder().list_recent().await?.data;
            output::output(&data, &config)?;
        }
        Some(("create", sub_matches)) => {
            let config = resolve_config(sub_matches);
            let folder_name = sub_matches
                .get_one::<String>("folder_name")
                .expect("folder_name is required");
            let parent = sub_matches.get_one::<String>("parent_folder_id");
            let parent = parent.and_then(|p| if p.is_empty() { None } else { Some(p.as_str()) });
            if config.dry_run {
                eprintln!("dry-run: would create folder \"{}\"", folder_name);
                return Ok(());
            }
            let data = client.folder().create(folder_name, parent).await?.data;
            output::output(&data, &config)?;
        }
        Some(("rename", sub_matches)) => {
            rename::execute(client, sub_matches).await?;
        }
        Some(("update", sub_matches)) => {
            let config = resolve_config(sub_matches);
            let folder_id = sub_matches
                .get_one::<String>("folder_id")
                .expect("folder_id is required");
            let new_name = sub_matches.get_one::<String>("new_name");
            let new_name =
                new_name.and_then(|n| if n.is_empty() { None } else { Some(n.as_str()) });
            let new_description = sub_matches.get_one::<String>("new_description");
            let new_description =
                new_description.and_then(|d| if d.is_empty() { None } else { Some(d.as_str()) });
            let new_color = sub_matches.get_one::<String>("new_color");
            let new_color =
                new_color.and_then(|c| if c.is_empty() { None } else { Some(c.as_str()) });
            if config.dry_run {
                eprintln!("dry-run: would update folder {}", folder_id);
                return Ok(());
            }
            let data = client
                .folder()
                .update(folder_id, new_name, new_description, new_color)
                .await?
                .data;
            output::output(&data, &config)?;
        }
        _ => {
            eprintln!("Error: No subcommand was used. Try: eagle-eye folder --help");
            std::process::exit(output::exit_code::USAGE);
        }
    }

    Ok(())
}
