use super::super::output::{self, resolve_config};
use super::super::stdin::read_ids_from_stdin;
use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("refresh-palette")
        .about("Refresh the color palette of an item")
        .arg(
            Arg::new("id")
                .value_name("ID")
                .help("Item ID (omit when using --stdin)"),
        )
        .arg(
            Arg::new("stdin")
                .long("stdin")
                .help("Read item IDs from stdin (JSON array or newline-delimited)")
                .action(clap::ArgAction::SetTrue),
        )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = resolve_config(matches);

    // Collect IDs from --stdin or positional arg
    let ids: Vec<String> = if matches.get_flag("stdin") {
        read_ids_from_stdin()?
    } else if let Some(id) = matches.get_one::<String>("id") {
        vec![id.clone()]
    } else {
        eprintln!("Error: provide item ID or use --stdin");
        std::process::exit(output::exit_code::USAGE);
    };

    if ids.is_empty() {
        eprintln!("Error: no item IDs provided");
        std::process::exit(output::exit_code::USAGE);
    }

    if config.dry_run {
        eprintln!(
            "dry-run: would refresh palette for {} item(s): {:?}",
            ids.len(),
            ids
        );
        return Ok(());
    }

    let mut successes: Vec<serde_json::Value> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for id in &ids {
        match client.item().refresh_palette(id).await {
            Ok(result) => {
                let val = serde_json::to_value(&result)?;
                successes.push(val);
            }
            Err(e) => {
                eprintln!("Error refreshing palette for {}: {}", id, e);
                failures.push(id.clone());
            }
        }
    }

    // Output results
    if successes.len() == 1 {
        output::output_value(&successes[0], &config)?;
    } else if !successes.is_empty() {
        let arr = serde_json::Value::Array(successes);
        output::output_value(&arr, &config)?;
    }

    // Exit code: 0 = all ok, 1 = all failed, 4 = partial
    if !failures.is_empty() {
        if failures.len() == ids.len() {
            std::process::exit(output::exit_code::ERROR);
        } else {
            std::process::exit(output::exit_code::PARTIAL);
        }
    }

    Ok(())
}
