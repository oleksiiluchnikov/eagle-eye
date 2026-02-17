use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("app")
        .about("Application")
        .arg(
            Arg::new("info")
                .short('i')
                .long("info")
                .help("Show application info")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .help("Show application version")
                .action(clap::ArgAction::SetTrue),
        )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = client.application().info().await?.data;

    if matches.get_flag("version") {
        println!("{}", data.version);
    } else if matches.get_flag("info") {
        println!("{}", serde_json::to_string_pretty(&data)?);
    } else {
        // Default: show info when no flag specified
        println!("{}", serde_json::to_string_pretty(&data)?);
    }
    Ok(())
}
