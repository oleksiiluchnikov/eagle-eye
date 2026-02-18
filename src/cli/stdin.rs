use std::io::{self, BufRead};

/// Read IDs from stdin: accepts a JSON array of strings or newline-delimited plain IDs.
pub fn read_ids_from_stdin() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut raw = String::new();
    for line in stdin.lock().lines() {
        let line = line?;
        raw.push_str(&line);
        raw.push('\n');
    }
    parse_ids_input(&raw)
}

/// Parse IDs from raw input: accepts a JSON array of strings or newline/null-delimited plain IDs.
pub fn parse_ids_input(raw: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let raw = raw.trim();

    if raw.is_empty() {
        return Ok(vec![]);
    }

    // Try JSON array first
    if raw.starts_with('[') {
        let ids: Vec<String> = serde_json::from_str(raw)?;
        return Ok(ids);
    }

    // Fall back to newline-delimited (also handles null-delimited via --print0 piping)
    let ids: Vec<String> = raw
        .split(['\n', '\0'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn parse_ids_single_id() {
        let ids = parse_ids_input("SINGLE_ID").unwrap();
        assert_eq!(ids, vec!["SINGLE_ID"]);
    }

    #[test]
    fn parse_ids_comma_in_json_not_split() {
        // JSON array should not be split by commas outside JSON parsing
        let input = r#"["a,b","c"]"#;
        let ids = parse_ids_input(input).unwrap();
        assert_eq!(ids, vec!["a,b", "c"]);
    }
}
