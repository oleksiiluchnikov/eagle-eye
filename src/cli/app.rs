use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("app").about("Application").arg(
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
    } else {
        println!("{}", serde_json::to_string_pretty(&data)?);
    }
    Ok(())
}
