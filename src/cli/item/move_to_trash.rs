use super::super::output::{self, resolve_config};
use super::super::stdin::read_ids_from_stdin;
use crate::lib::client::EagleClient;
use clap::{Arg, ArgMatches, Command};

pub fn build() -> Command {
    Command::new("move-to-trash")
        .about("Move items to trash")
        .arg(
            Arg::new("id")
                .value_name("ID")
                .help("Item ID(s) to move to trash (comma-separated for multiple)"),
        )
        .arg(
            Arg::new("force")
                .long("force")
                .help("Required safety flag for destructive operation")
                .action(clap::ArgAction::SetTrue),
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

    if !matches.get_flag("force") {
        eprintln!("Error: move-to-trash is destructive. Use --force to confirm.");
        std::process::exit(output::exit_code::USAGE);
    }

    // Collect IDs from --stdin or positional arg
    let ids: Vec<String> = if matches.get_flag("stdin") {
        read_ids_from_stdin()?
    } else if let Some(id_str) = matches.get_one::<String>("id") {
        id_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        eprintln!("Error: provide item ID(s) or use --stdin");
        std::process::exit(output::exit_code::USAGE);
    };

    if ids.is_empty() {
        eprintln!("Error: no item IDs provided");
        std::process::exit(output::exit_code::USAGE);
    }

    if config.dry_run {
        eprintln!(
            "dry-run: would move {} item(s) to trash: {:?}",
            ids.len(),
            ids
        );
        return Ok(());
    }

    let result = client.item().move_to_trash(&ids).await?;
    output::output(&result, &config)?;
    Ok(())
}

// Tests for parse_ids_input have been moved to cli::stdin module.
#[cfg(test)]
mod tests {
    use super::super::super::stdin::parse_ids_input;

    #[test]
    fn parse_ids_json_array() {
        let input = r#"["ID1","ID2","ID3"]"#;
        let ids = parse_ids_input(input).unwrap();
        assert_eq!(ids, vec!["ID1", "ID2", "ID3"]);
    }

    #[test]
    fn parse_ids_newline_delimited() {
        let input = "ID1\nID2\nID3\n";
        let ids = parse_ids_input(input).unwrap();
        assert_eq!(ids, vec!["ID1", "ID2", "ID3"]);
    }

    #[test]
    fn parse_ids_null_delimited() {
        let input = "ID1\0ID2\0ID3\0";
        let ids = parse_ids_input(input).unwrap();
        assert_eq!(ids, vec!["ID1", "ID2", "ID3"]);
    }

    #[test]
    fn parse_ids_empty_input() {
        let ids = parse_ids_input("").unwrap();
        assert!(ids.is_empty());
    }

    #[test]
    fn parse_ids_whitespace_only() {
        let ids = parse_ids_input("   \n  \n  ").unwrap();
        assert!(ids.is_empty());
    }

    #[test]
    fn parse_ids_mixed_blank_lines() {
        let input = "ID1\n\nID2\n\n";
        let ids = parse_ids_input(input).unwrap();
        assert_eq!(ids, vec!["ID1", "ID2"]);
    }

    #[test]
    fn parse_ids_json_array_with_whitespace() {
        let input = r#"  [ "A" , "B" ]  "#;
        let ids = parse_ids_input(input).unwrap();
        assert_eq!(ids, vec!["A", "B"]);
    }
}
