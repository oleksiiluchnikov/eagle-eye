use super::super::output::{self, resolve_config};
use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("move-to-trash")
        .about("Move items to trash")
        .arg(
            Arg::new("id")
                .value_name("ID")
                .help("Item ID(s) to move to trash (comma-separated for multiple)")
                .required(true),
        )
        .arg(
            Arg::new("force")
                .long("force")
                .help("Required safety flag for destructive operation")
                .action(clap::ArgAction::SetTrue),
        )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = resolve_config(matches);

    if !matches.get_flag("force") {
        eprintln!("Error: move-to-trash is destructive. Use --force to confirm.");
        std::process::exit(output::exit_code::USAGE);
    }

    if config.dry_run {
        let id_str = matches.get_one::<String>("id").expect("id is required");
        let ids: Vec<&str> = id_str.split(',').map(|s| s.trim()).collect();
        eprintln!(
            "dry-run: would move {} item(s) to trash: {:?}",
            ids.len(),
            ids
        );
        return Ok(());
    }

    let id_str = matches.get_one::<String>("id").expect("id is required");
    let ids: Vec<String> = id_str.split(',').map(|s| s.trim().to_string()).collect();

    let result = client.item().move_to_trash(&ids).await?;
    output::output(&result, &config)?;
    Ok(())
}
