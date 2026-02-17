use crate::lib::client::EagleClient;
use clap::ArgMatches;

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let folder_id = matches
        .get_one::<String>("folder_id")
        .expect("folder_id is required");
    let new_name = matches
        .get_one::<String>("new_name")
        .expect("new_name is required");

    let result = client.folder().rename(folder_id, new_name).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
