use crate::lib::client::EagleClient;
use clap::{Arg,ArgMatches,ArgAction, Command};
use crate::lib::types::{GetItemInfoParams, ItemInfoData};

pub fn build() -> Command {
    Command::new("info")
        .about("Get item info")
        .arg(
        Arg::new("id")
            .required(false)
            .value_name("ID")
            .help("Id of the file")
            .action(ArgAction::Set), //do not require a flag to be passed
    )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let raw_id: &str = matches.get_one::<String>("id").unwrap().as_str();
    println!("ID: {}", raw_id);

    let query_params: GetItemInfoParams = GetItemInfoParams {
        id: raw_id.to_string(),
    };

    let data: ItemInfoData = client.item().info(query_params).await?.data;
    println!("Item info: {:?}", data);
    Ok(())
}
