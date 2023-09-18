use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, ArgAction, Command};

pub struct App;

impl App {
    pub fn new() -> Self {
        App {}
    }
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
    ) -> Result<(), Box<dyn std::error::Error>> {
    let data = client.library().info().await?.data;

    if matches.get_flag("folders") {
        println!("{:?}", data.folders);
    }

    if matches.get_flag("smart-folders") {
        println!("{:?}", data.smart_folders);
    }

    if matches.get_flag("quick-access") {
        println!("{:?}", data.quick_access);
    }

    if matches.get_flag("tags-groups") {
        println!("{:?}", data.tags_groups);
    }

    if matches.get_flag("modification-time") {
        println!("{:?}", data.modification_time);
    }
    Ok(())
}

pub fn build() -> Command {

    Command::new("library")
        .about("Library")
        .subcommand(
            Command::new("info")
            .about("Library info")
            .arg(
                Arg::new("folders")
                .short('f')
                .long("folders")
                .help("Show folders")
                .action(clap::ArgAction::SetTrue)
                )
            .arg(
                Arg::new("smart_folders")
                .short('s')
                .long("smart-folders")
                .help("Show smart folders")
                .action(clap::ArgAction::SetTrue)
                )
            .arg(
                Arg::new("quick_access")
                .short('q')
                .long("quick-access")
                .help("Show quick access")
                .action(clap::ArgAction::SetTrue)
                )
            .arg(
                Arg::new("tags_groups")
                .short('t')
                .long("tags-groups")
                .help("Show tags groups")
                .action(clap::ArgAction::SetTrue)
                )
            .arg(
                Arg::new("modification_time")
                .short('m')
                .long("modification-time")
                .help("Show modification time")
                .action(clap::ArgAction::SetTrue)
                )
            )
            // .arg(
            //     Arg::new("history")
            //         .short('h')
            //         .long("history")
            //         )
            // .arg(
            //     Arg::new("switch")
            //         .short('s')
            //         .long("switch")
            //         )
            //         

}
