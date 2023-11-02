use crate::lib::client::EagleClient;
use clap::{Command, ArgMatches, Arg, ArgAction};
use crate::lib::types::Child;

// Arguments
pub mod args;

pub struct ListCommand {
    root: String,
    options: ListOptions,
}

pub struct ListOptions {
    recursive: bool,
    tree: bool,
    nesting_level: u8,
}

impl ListOptions {
    pub fn new() -> Self {
        ListOptions {
            recursive: false,
            tree: false,
            nesting_level: 0,
        }
    }
}

pub fn build() -> Command {
                Command::new("list")
                .about("List folders")

                .arg(
                    Arg::new("root")
                    .short('r')
                    .long("root")
                    .help("Specify root folder")
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
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
    ) -> Result<(), Box<dyn std::error::Error>> {

    let data: Vec<Child> = client.folder().list().await?.data;

    if matches.get_flag("tree") {
        args::tree::execute(&data, &ListOptions {
            recursive: matches.get_flag("recursive"),
            tree: matches.get_flag("tree"),
            nesting_level: 0,
        })?;
        return Ok(());
    }

    if matches.get_flag("recursive") {
        // let nesting_level = matches.get_one::<u8>("nesting-level")?;
        print_recursive(&data, 0);
        return Ok(());
    }
    match matches.subcommand() {
        Some(("tree", matches)) => {
            args::tree::execute(&data, &ListOptions {
                recursive: matches.get_flag("recursive"),
                tree: matches.get_flag("tree"),
                nesting_level: 0,
            })?;
        }
        // Some(("recursive", matches)) => {
        //     args::recursive::execute(&data, &ListOptions {
        //         recursive: matches.get_flag("recursive"),
        //         tree: matches.get_flag("tree"),
        //         nesting_level: 0,
        //     })?;
        // }
        _ => {
            // print 'folders list' output
            for child in data {
                println!("{}", child.name);
            }
        }
    }

    Ok(())
}

fn print_recursive(data: &Vec<Child>, mut nesting_level: u8) {
    for child in data {
        println!("{}", child.name);
        if child.children.len() > 0 {
            nesting_level += 1;
            print_recursive(&child.children, nesting_level);
        }
    }
}


// Recursive function to find duplicate folder names among siblings (having the same parent)
fn find_duplicates(data: &Vec<Child>, duplicate_folder_names: &mut Vec<String>) {
    todo!()
}

