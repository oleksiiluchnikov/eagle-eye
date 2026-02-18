use super::super::output::{self, resolve_config};
use super::super::stdin::read_ids_from_stdin;
use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("update")
        .about("Update an item's properties")
        .arg(
            Arg::new("id")
                .value_name("ID")
                .help("Item ID to update (omit when using --stdin)"),
        )
        .arg(
            Arg::new("tags")
                .long("tags")
                .value_name("TAGS")
                .help("Comma-separated tags (replaces existing tags)"),
        )
        .arg(
            Arg::new("annotation")
                .long("annotation")
                .value_name("TEXT")
                .help("Annotation text"),
        )
        .arg(
            Arg::new("url")
                .long("url")
                .value_name("URL")
                .help("Source URL"),
        )
        .arg(
            Arg::new("star")
                .long("star")
                .value_name("RATING")
                .help("Star rating (0-5)")
                .value_parser(clap::value_parser!(u8)),
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

    let tags: Option<Vec<String>> = matches
        .get_one::<String>("tags")
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
    let annotation = matches.get_one::<String>("annotation").map(|s| s.as_str());
    let url = matches.get_one::<String>("url").map(|s| s.as_str());
    let star = matches.get_one::<u8>("star").copied();

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
        eprintln!("dry-run: would update {} item(s): {:?}", ids.len(), ids);
        return Ok(());
    }

    let mut successes: Vec<serde_json::Value> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for id in &ids {
        match client
            .item()
            .update(id, tags.as_deref(), annotation, url, star)
            .await
        {
            Ok(result) => {
                let val = serde_json::to_value(&result.data)?;
                successes.push(val);
            }
            Err(e) => {
                eprintln!("Error updating {}: {}", id, e);
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
