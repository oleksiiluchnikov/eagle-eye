use super::super::output::{self, resolve_config};
use crate::lib::client::EagleClient;
use crate::lib::types::{GetItemInfoParams, ItemInfoData};
use clap::{Arg, ArgAction, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("info").about("Get item info").arg(
        Arg::new("id")
            .required(true)
            .value_name("ID")
            .help("Id of the file")
            .action(ArgAction::Set),
    )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = resolve_config(matches);
    let raw_id = matches.get_one::<String>("id").expect("id is required");

    let query_params = GetItemInfoParams {
        id: raw_id.to_string(),
    };

    let data: ItemInfoData = client.item().info(query_params).await?.data;
    output::output(&data, &config)?;
    Ok(())
}
