use super::super::output;
use crate::lib::client::EagleClient;
use crate::lib::types::{GetItemThumbnailParams, ItemThumbnailData};
use clap::{Arg, ArgAction, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("thumbnail").about("Get item thumbnail").arg(
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
    let raw_id = matches.get_one::<String>("id").expect("id is required");

    let query_params = GetItemThumbnailParams {
        id: raw_id.to_string(),
    };
    let thumbnail_path: ItemThumbnailData = client.item().thumbnail(query_params).await?.data;
    let path = percent_encoding::percent_decode_str(&thumbnail_path).decode_utf8()?;
    output::output_plain(&path);
    Ok(())
}
