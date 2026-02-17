use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("refresh-palette")
        .about("Refresh the color palette of an item")
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
    let id = match matches.get_one::<String>("id") {
        Some(id) => id,
        None => {
            println!("No ID provided");
            return Ok(());
        }
    };

    let result = client.item().refresh_palette(id).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
