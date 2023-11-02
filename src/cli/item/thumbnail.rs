use crate::lib::client::EagleClient;
use clap::{Arg,ArgMatches,ArgAction, Command};
use crate::lib::types::{GetItemThumbnailParams, ItemThumbnailData};

pub fn build() -> Command {
    Command::new("thumbnail").about("Get item thumbnail").arg(
        Arg::new("id")
            .required(false)
            .value_name("ID")
            .action(ArgAction::Set), //do not require a flag to be passed
    )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let raw_id: &str = matches.get_one::<String>("id").unwrap().as_str();
    // let id = GetItemListParams {
    //     id=raw_id
    // }
    let query_params: GetItemThumbnailParams = GetItemThumbnailParams {
        id: raw_id.to_string(),
    };
    let thumbnail_path: ItemThumbnailData = client.item().thumbnail(query_params).await?.data;
    let path = percent_encoding::percent_decode_str(&thumbnail_path).decode_utf8()?;
    println!("{}", path);
    Ok(())
}
