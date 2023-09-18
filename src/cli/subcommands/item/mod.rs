use clap::{Arg, ArgMatches, ArgAction, Command};

pub fn build() -> Command {
                Command::new("item")
            .about("Item")
            .arg(
                Arg::new("add")
                .help("Add item")
                // TODO: Add arguments
                ) 
            .arg(
                Arg::new("info")
                .short('i')
                .long("info")
                .help("Show item info")
                )
            .arg(
                Arg::new("thumbnail")
                .short('t')
                .long("thumbnail")
                .help("Show item thumbnail")
                )
            .arg(
                Arg::new("list")
                .short('l')
                .long("list")
                .help("List items")
                )

}
