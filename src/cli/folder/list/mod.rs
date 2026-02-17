use crate::lib::client::EagleClient;
use crate::lib::types::Child;
use clap::{Arg, ArgAction, ArgMatches, Command};

// Arguments
pub mod args;

#[derive(Default)]
pub struct ListOptions {
    pub(crate) recursive: bool,
    /// Maximum depth to recurse into. `None` means unlimited.
    pub(crate) max_depth: Option<usize>,
}

pub fn build() -> Command {
    Command::new("list")
        .about("List folders")
        .arg(
            Arg::new("root")
                .short('r')
                .long("root")
                .help("Specify root folder"),
        )
        .arg(
            Arg::new("tree")
                .short('t')
                .long("tree")
                .help("Show folder tree")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("max_depth")
                .short('n')
                .long("max-depth")
                .value_name("DEPTH")
                .help("Limit tree depth (0 = immediate children only)")
                .num_args(1)
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("recursive")
                .short('R')
                .long("recursive")
                .help("Show folder tree recursively")
                .action(ArgAction::SetTrue),
        )
}

pub async fn execute(
    client: &EagleClient,
    matches: &ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    let data: Vec<Child> = client.folder().list().await?.data;

    let max_depth = matches.get_one::<usize>("max_depth").copied();

    if matches.get_flag("tree") {
        args::tree::execute(
            &data,
            &ListOptions {
                recursive: matches.get_flag("recursive"),
                max_depth,
            },
        )?;
        return Ok(());
    }

    if matches.get_flag("recursive") {
        print_recursive(&data, 0);
        return Ok(());
    }

    for child in data {
        println!("{}", child.name);
    }

    Ok(())
}

fn print_recursive(data: &[Child], nesting_level: u8) {
    print_recursive_to_writer(data, nesting_level, &mut std::io::stdout());
}

fn print_recursive_to_writer<W: std::io::Write>(data: &[Child], nesting_level: u8, writer: &mut W) {
    for child in data {
        let indent = "  ".repeat(nesting_level as usize);
        writeln!(writer, "{}{}", indent, child.name).unwrap();
        if !child.children.is_empty() {
            print_recursive_to_writer(&child.children, nesting_level + 1, writer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::types::Child;

    fn create_test_child(name: &str, children: Vec<Child>) -> Child {
        Child {
            id: format!("id-{}", name),
            name: name.to_string(),
            images: None,
            folders: None,
            modification_time: 0,
            editable: None,
            tags: vec![],
            children,
            is_expand: None,
            size: None,
            vstype: None,
            styles: None,
            is_visible: None,
            index: None,
            new_folder_name: None,
            image_count: None,
            descendant_image_count: None,
            pinyin: None,
            extend_tags: None,
            covers: None,
            parent: None,
        }
    }

    #[test]
    fn print_recursive_single_folder() {
        let folders = vec![create_test_child("Root", vec![])];
        let mut output = Vec::new();
        print_recursive_to_writer(&folders, 0, &mut output);
        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "Root\n");
    }

    #[test]
    fn print_recursive_nested_folders() {
        let child = create_test_child("Child", vec![]);
        let parent = create_test_child("Parent", vec![child]);
        let folders = vec![parent];

        let mut output = Vec::new();
        print_recursive_to_writer(&folders, 0, &mut output);
        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "Parent\n  Child\n");
    }

    #[test]
    fn print_recursive_deeply_nested() {
        let grandchild = create_test_child("Grandchild", vec![]);
        let child = create_test_child("Child", vec![grandchild]);
        let parent = create_test_child("Parent", vec![child]);
        let folders = vec![parent];

        let mut output = Vec::new();
        print_recursive_to_writer(&folders, 0, &mut output);
        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "Parent\n  Child\n    Grandchild\n");
    }

    #[test]
    fn print_recursive_multiple_siblings() {
        let folders = vec![
            create_test_child("Folder A", vec![]),
            create_test_child("Folder B", vec![]),
            create_test_child("Folder C", vec![]),
        ];

        let mut output = Vec::new();
        print_recursive_to_writer(&folders, 0, &mut output);
        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "Folder A\nFolder B\nFolder C\n");
    }

    #[test]
    fn print_recursive_with_initial_indent() {
        let folders = vec![create_test_child(
            "Root",
            vec![create_test_child("Child", vec![])],
        )];

        let mut output = Vec::new();
        print_recursive_to_writer(&folders, 2, &mut output);
        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "    Root\n      Child\n");
    }

    #[test]
    fn print_recursive_complex_tree() {
        // Parent
        // ├── Child 1
        // │   └── Grandchild
        // └── Child 2
        let grandchild = create_test_child("Grandchild", vec![]);
        let child1 = create_test_child("Child 1", vec![grandchild]);
        let child2 = create_test_child("Child 2", vec![]);
        let parent = create_test_child("Parent", vec![child1, child2]);

        let folders = vec![parent];

        let mut output = Vec::new();
        print_recursive_to_writer(&folders, 0, &mut output);
        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "Parent\n  Child 1\n    Grandchild\n  Child 2\n");
    }
}
