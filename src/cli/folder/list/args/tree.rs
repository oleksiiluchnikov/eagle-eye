use crate::cli::folder::list::ListOptions;
use crate::lib::types::*;
use clap::builder::styling::AnsiColor;

pub fn print_folder_tree(folder: Option<&Child>, indent: &str, last: bool, depth: usize) {
    let colors = [
        (AnsiColor::Red, "31"),
        (AnsiColor::Green, "32"),
        (AnsiColor::Yellow, "33"),
        (AnsiColor::Blue, "34"),
        (AnsiColor::Magenta, "35"),
        (AnsiColor::Cyan, "36"),
    ];

    let (_, color_code) = colors[depth % colors.len()];

    let (corner, vertical_line) = if last {
        ("╰── ", "    ")
    } else {
        ("├── ", "│   ")
    };

    if let Some(folder) = folder {
        let formatted_name = format!("\x1b[{}m{}\x1b[0m", color_code, folder.name);
        let formatted_corner = format!("\x1b[{}m{}\x1b[0m", color_code, corner);

        println!("{}{}{}", indent, formatted_corner, formatted_name);

        let mut children_iter = folder.children.iter();
        let child_count = folder.children.len();
        let new_depth = depth + 1;

        for i in 0..child_count {
            if let Some(child) = children_iter.next() {
                let new_indent = format!("{}{}", indent, vertical_line);
                print_folder_tree(Some(child), &new_indent, i == child_count - 1, new_depth);
            }
        }
    } else {
        println!("\x1b[31mNo folder was provided\x1b[0m");
    }
}

pub fn execute(data: &[Child], options: &ListOptions) -> Result<(), Box<dyn std::error::Error>> {
    if options.recursive {
        for folder in data {
            println!("{}", folder.name);
            let initial_indent = "    ";
            if !folder.children.is_empty() {
                for (j, child) in folder.children.iter().enumerate() {
                    print_folder_tree(
                        Some(child),
                        initial_indent,
                        j == folder.children.len() - 1,
                        0,
                    );
                }
            }
        }
    } else {
        for folder in data {
            println!("{}", folder.name);
        }
    }
    Ok(())
}
