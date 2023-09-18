use crate::lib::client::EagleClient;
use clap::ArgMatches;
use clap::{Arg, Command};

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
    let data = client.application().info().await?.data;
    println!("{:?}", data);
    Ok(())
}

pub fn build() -> Command {
    Command::new("app")
        .about("Application")
        .arg(
            Arg::new("info")
            .short('i')
            .long("info")
            .help("Show application info")
            .action(clap::ArgAction::SetTrue)
            .default_value("true")
            )
}
