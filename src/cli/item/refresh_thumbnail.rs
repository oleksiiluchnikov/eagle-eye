use super::super::output::{self, resolve_format};
use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("refresh-thumbnail")
        .about("Refresh the thumbnail of an item")
        .arg(
            Arg::new("id")
                .value_name("ID")
                .help("Item ID")
                .required(true),
        )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let fmt = resolve_format(matches);
    let id = matches.get_one::<String>("id").expect("id is required");

    let result = client.item().refresh_thumbnail(id).await?;
    output::output(&result, &fmt)?;
    Ok(())
}
