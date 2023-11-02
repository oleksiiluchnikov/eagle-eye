use crate::lib;
use clap::ArgMatches;
use std::collections::HashMap;

async fn append_duplicate_suffix_to_each_duplicate_name(folders: &Vec<&serde_json::Value>, root_folder: &serde_json::Value, name_count: &mut HashMap<String, usize>) {
    // let mut initial_name_count = HashMap::new();
    // let mut duplicates = Vec::new();
    // let mut local_duplicates = HashMap::new();
    // list_duplicate_folders(&folders, &root_folder, &mut initial_name_count, &mut duplicates, "".to_string(), &mut local_duplicates);
    //
    // for duplicate in duplicates {
    //     println!("eagle://folder/{} - {}", duplicate.id, duplicate.name);
    //     // Rename the duplicate folder to duplicate.name + " DUPLICATE"
    //     let new_name = format!("{} DUPLICATE", duplicate.name);
    //     if duplicate.name.contains("DUPLICATE") {
    //         println!("{} is already a duplicate", duplicate.name);
    //         continue;
    //     }
    //     rename_folder(&client, &duplicate.id, &new_name).await?;
    // }

    // fetch_duplicate_folders(&folders, &root_folder, name_count).await;

}

pub async fn execute(
    client: lib::client::EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let id = matches.get_one::<String>("ID");
    let name = matches.get_one::<String>("NAME");

    // Convert id to &str
    let id = match id {
        Some(id) => id,
        None => {
            println!("No ID was provided");
            return Ok(());
        }
    };

    // Convert name to &str
    let name = match name {
        Some(name) => name,
        None => {
            println!("No name was provided");
            return Ok(());
        }
    };

    // Your logic can go here for using the 'id' and 'name' variables

    Ok(())
}
