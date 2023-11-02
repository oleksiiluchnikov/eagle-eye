use crate::lib::client::EagleClient;
use crate::lib::types::{GetItemListParams, ItemListData, Order};
use clap::{Arg, ArgAction, ArgMatches, Command};
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
                .num_args(1),
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
        todo!()
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

    let library_data = client.library().info().await?.data;
    let library_path = Path::new(&library_data.library.path).join("images");

    let thumbnails_flag = matches.get_flag("thumbnails");
    let url_flag = matches.get_one::<String>("url").unwrap().len() > 0;
    let url_keyword = matches.get_one::<String>("url").unwrap();

    let items: Vec<ItemListData> = client.item().list(query_params).await?.data;

    let paths: Vec<_> = items
        .par_iter()
        .filter(|item| {
            if url_flag && url_keyword.len() > 0 {
                item.url.contains(url_keyword)
            } else {
                true
            }
        })
        .map(|item| {
            // let item_dir_name = &item.id + ".info";
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

    for path in &paths {
        println!("{}", path.display());
    }

    Ok(())
}
