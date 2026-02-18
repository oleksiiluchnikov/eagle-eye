use super::output::{self, output_plain, resolve_config};
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
        output_plain(&data.version);
    } else {
        let config = resolve_config(matches);
        output::output(&data, &config)?;
    }
    Ok(())
}
