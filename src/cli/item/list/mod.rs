use super::super::output::{self, resolve_format};
use crate::lib::client::EagleClient;
use crate::lib::types::{GetItemListParams, ItemListData, Order};
use clap::{Arg, ArgMatches, Command};
use rayon::prelude::*;
use std::path::Path;

pub fn build() -> Command {
    Command::new("list")
        .about("List items")
        .arg(
            Arg::new("length")
                .short('l')
                .value_name("LENGTH")
                .long("length")
                .help("Get the length of the list")
                .num_args(0),
        )
        .arg(
            Arg::new("limit")
                .short('n')
                .long("limit")
                .value_name("LIMIT")
                .help("Limit the number of items")
                .num_args(1)
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("offset")
                .short('O')
                .long("offset")
                .value_name("OFFSET")
                .help("Offset the list")
                .num_args(1)
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("order_by")
                .short('o')
                .long("order-by")
                .value_name("ORDER-BY")
                .help("The sorting order")
                .num_args(1)
                .allow_hyphen_values(true),
        )
        .arg(
            Arg::new("keyword")
                .short('k')
                .value_name("KEYWORD")
                .long("keyword")
                .help("Filter by keyword that in filename")
                .num_args(1),
        )
        .arg(
            Arg::new("ext")
                .short('e')
                .value_name("EXTENSION")
                .long("ext")
                .help("Filter by extension")
                .num_args(1),
        )
        .arg(
            Arg::new("tags")
                .short('t')
                .long("tags")
                .value_name("TAG")
                .help("Filter by tags. Comma separated. It works like OR")
                .num_args(1)
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            Arg::new("folders")
                .short('f')
                .long("folders")
                .value_name("FOLDER-ID")
                .help("Filter by folders ids. Comma separated. It works like OR")
                .num_args(1)
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            Arg::new("thumbnails")
                .short('T')
                .long("thumbnails")
                .value_name("THUMBNAILS")
                .help("Get the list of path to thumbnails")
                .num_args(0),
        )
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .value_name("KEYWORD")
                .help("Get the list of items with url")
                .num_args(1)
                .default_value(""),
        )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut query_params: GetItemListParams = GetItemListParams::new();

    if let Some(limit) = matches.get_one::<usize>("limit") {
        query_params.limit = Some(*limit);
    }

    if let Some(offset) = matches.get_one::<usize>("offset") {
        query_params.offset = Some(*offset);
    }

    if let Some(order_by) = matches.get_one::<String>("order_by") {
        match parse_order(order_by) {
            Some(order) => {
                query_params.order_by = Some(order);
            }
            None => {
                eprintln!("Error: Unknown order: '{}'. Valid values: MANUAL, CREATEDATE, -CREATEDATE, BTIME, MTIME, FILESIZE, -FILESIZE, NAME, -NAME, RESOLUTION, -RESOLUTION", order_by);
                std::process::exit(output::exit_code::USAGE);
            }
        }
    }

    if let Some(keyword) = matches.get_one::<String>("keyword") {
        query_params.keyword = Some(keyword.to_owned());
    }

    if let Some(ext) = matches.get_one::<String>("ext") {
        query_params.ext = Some(ext.to_owned());
    }

    if let Some(tags) = matches.get_one::<String>("tags") {
        query_params.tags = Some(tags.to_owned());
    }

    if let Some(folders) = matches.get_one::<String>("folders") {
        query_params.folders = Some(folders.to_owned());
    }

    let items: Vec<ItemListData> = client.item().list(query_params).await?.data;

    // When an explicit output format is requested (--json / --output),
    // output the raw item data instead of derived file paths.
    let explicit_format = matches.get_flag("json") || matches.get_one::<String>("output").is_some();
    if explicit_format {
        let fmt = resolve_format(matches);
        let length_flag = matches.get_flag("length");
        if length_flag {
            output::output_plain(&items.len().to_string());
        } else {
            output::output(&items, &fmt)?;
        }
        return Ok(());
    }

    // Default mode: output derived file paths (human-friendly).
    let library_data = client.library().info().await?.data;
    let library_path = Path::new(&library_data.library.path).join("images");

    let thumbnails_flag = matches.get_flag("thumbnails");
    let url_keyword = matches
        .get_one::<String>("url")
        .map(|s| s.as_str())
        .unwrap_or("");
    let url_flag = !url_keyword.is_empty();

    let paths: Vec<_> = items
        .par_iter()
        .filter(|item| {
            if url_flag && !url_keyword.is_empty() {
                item.url.contains(url_keyword)
            } else {
                true
            }
        })
        .map(|item| {
            let item_id = String::from(&item.id);
            let item_dir_name = item_id + ".info";
            let basename = &item.name;

            if thumbnails_flag {
                let thumbnail_filename = basename.to_owned() + "_thumbnail" + ".png";
                let potential_path = library_path.join(&item_dir_name).join(&thumbnail_filename);

                if potential_path.exists() {
                    return potential_path;
                }
            }

            let filename = basename.to_owned() + "." + item.ext.as_str();
            library_path.join(item_dir_name).join(filename)
        })
        .collect();

    let length_flag = matches.get_flag("length");

    if length_flag {
        println!("{}", paths.len());
    } else {
        for path in &paths {
            println!("{}", path.display());
        }
    }

    Ok(())
}

/// Parse an order string into an Order enum variant.
/// Returns None if the order string is not recognized.
pub fn parse_order(order_str: &str) -> Option<Order> {
    match order_str.to_uppercase().as_str() {
        "MANUAL" => Some(Order::MANUAL),
        "CREATEDATE" => Some(Order::CREATEDATE),
        "-CREATEDATE" | "CREATEDATEDESC" => Some(Order::CREATEDATEDESC),
        "BTIME" => Some(Order::BTIME),
        "MTIME" => Some(Order::MTIME),
        "FILESIZE" => Some(Order::FILESIZE),
        "-FILESIZE" | "FILESIZEREVERSE" => Some(Order::FILESIZEREVERSE),
        "NAME" => Some(Order::NAME),
        "-NAME" | "NAMEREVERSE" => Some(Order::NAMEREVERSE),
        "RESOLUTION" => Some(Order::RESOLUTION),
        "-RESOLUTION" | "RESOLUTIONREVERSE" => Some(Order::RESOLUTIONREVERSE),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Order Parsing Tests
    // =========================================================================

    #[test]
    fn parse_order_manual() {
        assert!(matches!(parse_order("MANUAL").unwrap(), Order::MANUAL));
        assert!(matches!(parse_order("manual").unwrap(), Order::MANUAL));
    }

    #[test]
    fn parse_order_createdate() {
        assert!(matches!(
            parse_order("CREATEDATE").unwrap(),
            Order::CREATEDATE
        ));
        assert!(matches!(
            parse_order("createdate").unwrap(),
            Order::CREATEDATE
        ));
    }

    #[test]
    fn parse_order_createdate_desc() {
        assert!(matches!(
            parse_order("-CREATEDATE").unwrap(),
            Order::CREATEDATEDESC
        ));
        assert!(matches!(
            parse_order("CREATEDATEDESC").unwrap(),
            Order::CREATEDATEDESC
        ));
        assert!(matches!(
            parse_order("createdatedesc").unwrap(),
            Order::CREATEDATEDESC
        ));
    }

    #[test]
    fn parse_order_btime() {
        assert!(matches!(parse_order("BTIME").unwrap(), Order::BTIME));
        assert!(matches!(parse_order("btime").unwrap(), Order::BTIME));
    }

    #[test]
    fn parse_order_mtime() {
        assert!(matches!(parse_order("MTIME").unwrap(), Order::MTIME));
        assert!(matches!(parse_order("mtime").unwrap(), Order::MTIME));
    }

    #[test]
    fn parse_order_filesize() {
        assert!(matches!(parse_order("FILESIZE").unwrap(), Order::FILESIZE));
        assert!(matches!(parse_order("filesize").unwrap(), Order::FILESIZE));
    }

    #[test]
    fn parse_order_filesize_reverse() {
        assert!(matches!(
            parse_order("-FILESIZE").unwrap(),
            Order::FILESIZEREVERSE
        ));
        assert!(matches!(
            parse_order("FILESIZEREVERSE").unwrap(),
            Order::FILESIZEREVERSE
        ));
        assert!(matches!(
            parse_order("filesizereverse").unwrap(),
            Order::FILESIZEREVERSE
        ));
    }

    #[test]
    fn parse_order_name() {
        assert!(matches!(parse_order("NAME").unwrap(), Order::NAME));
        assert!(matches!(parse_order("name").unwrap(), Order::NAME));
    }

    #[test]
    fn parse_order_name_reverse() {
        assert!(matches!(parse_order("-NAME").unwrap(), Order::NAMEREVERSE));
        assert!(matches!(
            parse_order("NAMEREVERSE").unwrap(),
            Order::NAMEREVERSE
        ));
        assert!(matches!(
            parse_order("namereverse").unwrap(),
            Order::NAMEREVERSE
        ));
    }

    #[test]
    fn parse_order_resolution() {
        assert!(matches!(
            parse_order("RESOLUTION").unwrap(),
            Order::RESOLUTION
        ));
        assert!(matches!(
            parse_order("resolution").unwrap(),
            Order::RESOLUTION
        ));
    }

    #[test]
    fn parse_order_resolution_reverse() {
        assert!(matches!(
            parse_order("-RESOLUTION").unwrap(),
            Order::RESOLUTIONREVERSE
        ));
        assert!(matches!(
            parse_order("RESOLUTIONREVERSE").unwrap(),
            Order::RESOLUTIONREVERSE
        ));
        assert!(matches!(
            parse_order("resolutionreverse").unwrap(),
            Order::RESOLUTIONREVERSE
        ));
    }

    #[test]
    fn parse_order_unknown() {
        assert!(parse_order("UNKNOWN").is_none());
        assert!(parse_order("invalid").is_none());
        assert!(parse_order("").is_none());
        assert!(parse_order("file").is_none());
    }

    #[test]
    fn parse_order_mixed_case() {
        // Test that mixed case is handled properly
        assert!(matches!(parse_order("FileSize").unwrap(), Order::FILESIZE));
        assert!(matches!(
            parse_order("CreateDate").unwrap(),
            Order::CREATEDATE
        ));
        assert!(matches!(
            parse_order("-filesize").unwrap(),
            Order::FILESIZEREVERSE
        ));
    }
}
