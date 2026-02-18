use super::super::output::{self, resolve_config};
use crate::lib::client::EagleClient;
use clap::ArgMatches;

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = resolve_config(matches);
    let folder_id = matches
        .get_one::<String>("folder_id")
        .expect("folder_id is required");
    let new_name = matches
        .get_one::<String>("new_name")
        .expect("new_name is required");

    if config.dry_run {
        eprintln!(
            "dry-run: would rename folder {} to \"{}\"",
            folder_id, new_name
        );
        return Ok(());
    }

    let data = client.folder().rename(folder_id, new_name).await?.data;
    output::output(&data, &config)?;
    Ok(())
}
