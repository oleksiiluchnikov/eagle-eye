use crate::lib::client::EagleClient;
use crate::lib::types::Child;
use clap::{Arg, ArgAction, ArgMatches, Command};

// Arguments
pub mod args;

#[derive(Default)]
pub struct ListOptions {
    pub(crate) recursive: bool,
    _tree: bool,
    _nesting_level: u8,
}

impl ListOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn build() -> Command {
    Command::new("list")
        .about("List folders")
        .arg(
            Arg::new("root")
                .short('r')
                .long("root")
                .help("Specify root folder"),
        )
        .arg(
            Arg::new("tree")
                .short('t')
                .long("tree")
                .help("Show folder tree")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("nesting_level")
                .short('n')
                .long("nesting-level")
                .help("Specify nesting level")
                .required(false)
                .default_value("0"),
        )
        .arg(
            Arg::new("recursive")
                .short('R')
                .long("recursive")
                .help("Show folder tree recursively")
                .action(ArgAction::SetTrue),
        )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let data: Vec<Child> = client.folder().list().await?.data;

    if matches.get_flag("tree") {
        args::tree::execute(
            &data,
            &ListOptions {
                recursive: matches.get_flag("recursive"),
                ..Default::default()
            },
        )?;
        return Ok(());
    }

    if matches.get_flag("recursive") {
        print_recursive(&data, 0);
        return Ok(());
    }
    match matches.subcommand() {
        Some(("tree", matches)) => {
            args::tree::execute(
                &data,
                &ListOptions {
                    recursive: matches.get_flag("recursive"),
                    ..Default::default()
                },
            )?;
        }

        _ => {
            for child in data {
                println!("{}", child.name);
            }
        }
    }

    Ok(())
}

fn print_recursive(data: &[Child], nesting_level: u8) {
    for child in data {
        let indent = "  ".repeat(nesting_level as usize);
        println!("{}{}", indent, child.name);
        if !child.children.is_empty() {
            print_recursive(&child.children, nesting_level + 1);
        }
    }
}
