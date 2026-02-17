use crate::cli::folder::list::ListOptions;
use crate::lib::types::Child;
use std::io::{IsTerminal, Write};

const DEPTH_COLORS: &[&str] = &["31", "32", "33", "34", "35", "36"];

/// Write a folder tree node and recurse into its children.
///
/// - `folder`: The folder to print.
/// - `indent`: Accumulated prefix string for this depth.
/// - `last`: Whether this node is the last sibling.
/// - `depth`: Current depth (0-based), used for color cycling.
/// - `max_depth`: Optional maximum depth to recurse into (None = unlimited).
/// - `color`: Whether to emit ANSI color codes.
/// - `writer`: Output destination.
fn write_folder_tree<W: Write>(
    folder: &Child,
    indent: &str,
    last: bool,
    depth: usize,
    max_depth: Option<usize>,
    color: bool,
    writer: &mut W,
) {
    let (connector, continuation) = if last {
        ("\u{2570}\u{2500}\u{2500} ", "    ")
    } else {
        ("\u{251c}\u{2500}\u{2500} ", "\u{2502}   ")
    };

    if color {
        let code = DEPTH_COLORS[depth % DEPTH_COLORS.len()];
        writeln!(
            writer,
            "{}\x1b[{code}m{connector}\x1b[0m\x1b[{code}m{}\x1b[0m",
            indent, folder.name,
        )
        .ok();
    } else {
        writeln!(writer, "{}{}{}", indent, connector, folder.name).ok();
    }

    let should_recurse = match max_depth {
        Some(max) => depth < max,
        None => true,
    };

    if should_recurse {
        let child_count = folder.children.len();
        for (i, child) in folder.children.iter().enumerate() {
            let new_indent = format!("{}{}", indent, continuation);
            write_folder_tree(
                child,
                &new_indent,
                i == child_count - 1,
                depth + 1,
                max_depth,
                color,
                writer,
            );
        }
    }
}

/// Render a folder list as a tree to the given writer.
pub fn write_tree<W: Write>(data: &[Child], options: &ListOptions, color: bool, writer: &mut W) {
    for folder in data {
        if color {
            let code = DEPTH_COLORS[0];
            writeln!(writer, "\x1b[{code};1m{}\x1b[0m", folder.name).ok();
        } else {
            writeln!(writer, "{}", folder.name).ok();
        }

        if options.recursive && !folder.children.is_empty() {
            let child_count = folder.children.len();
            for (i, child) in folder.children.iter().enumerate() {
                write_folder_tree(
                    child,
                    "",
                    i == child_count - 1,
                    0,
                    options.max_depth,
                    color,
                    writer,
                );
            }
        }
    }
}

/// Entry point called from the CLI handler. Detects TTY and writes to stdout.
pub fn execute(data: &[Child], options: &ListOptions) -> Result<(), Box<dyn std::error::Error>> {
    let stdout = std::io::stdout();
    let color = stdout.is_terminal();
    let mut writer = stdout.lock();
    write_tree(data, options, color, &mut writer);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::types::Child;

    fn child(name: &str, children: Vec<Child>) -> Child {
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

    fn opts(recursive: bool, max_depth: Option<usize>) -> ListOptions {
        ListOptions {
            recursive,
            max_depth,
        }
    }

    fn render(data: &[Child], options: &ListOptions) -> String {
        let mut buf = Vec::new();
        write_tree(data, options, false, &mut buf);
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn non_recursive_shows_names_only() {
        let data = vec![
            child("Alpha", vec![child("Hidden", vec![])]),
            child("Beta", vec![]),
        ];
        let out = render(&data, &opts(false, None));
        assert_eq!(out, "Alpha\nBeta\n");
    }

    #[test]
    fn recursive_single_root_no_children() {
        let data = vec![child("Root", vec![])];
        let out = render(&data, &opts(true, None));
        assert_eq!(out, "Root\n");
    }

    #[test]
    fn recursive_single_child() {
        let data = vec![child("Root", vec![child("Child", vec![])])];
        let out = render(&data, &opts(true, None));
        assert_eq!(out, "Root\n\u{2570}\u{2500}\u{2500} Child\n");
    }

    #[test]
    fn recursive_two_children() {
        let data = vec![child("Root", vec![child("A", vec![]), child("B", vec![])])];
        let out = render(&data, &opts(true, None));
        assert_eq!(
            out,
            "Root\n\u{251c}\u{2500}\u{2500} A\n\u{2570}\u{2500}\u{2500} B\n"
        );
    }

    #[test]
    fn recursive_nested_tree() {
        let data = vec![child(
            "Root",
            vec![
                child("A", vec![child("A1", vec![]), child("A2", vec![])]),
                child("B", vec![]),
            ],
        )];
        let out = render(&data, &opts(true, None));
        let expected = "\
Root
\u{251c}\u{2500}\u{2500} A
\u{2502}   \u{251c}\u{2500}\u{2500} A1
\u{2502}   \u{2570}\u{2500}\u{2500} A2
\u{2570}\u{2500}\u{2500} B
";
        assert_eq!(out, expected);
    }

    #[test]
    fn max_depth_zero_hides_children() {
        let data = vec![child("Root", vec![child("A", vec![child("Deep", vec![])])])];
        let out = render(&data, &opts(true, Some(0)));
        // depth 0 means don't recurse past the immediate children display
        assert_eq!(out, "Root\n\u{2570}\u{2500}\u{2500} A\n");
    }

    #[test]
    fn max_depth_one_shows_one_level() {
        let data = vec![child("Root", vec![child("A", vec![child("Deep", vec![])])])];
        let out = render(&data, &opts(true, Some(1)));
        let expected = "\
Root
\u{2570}\u{2500}\u{2500} A
    \u{2570}\u{2500}\u{2500} Deep
";
        assert_eq!(out, expected);
    }

    #[test]
    fn multiple_roots() {
        let data = vec![
            child("Root1", vec![child("C1", vec![])]),
            child("Root2", vec![child("C2", vec![])]),
        ];
        let out = render(&data, &opts(true, None));
        let expected = "\
Root1
\u{2570}\u{2500}\u{2500} C1
Root2
\u{2570}\u{2500}\u{2500} C2
";
        assert_eq!(out, expected);
    }

    #[test]
    fn color_output_contains_ansi_codes() {
        let data = vec![child("Root", vec![child("Kid", vec![])])];
        let mut buf = Vec::new();
        write_tree(&data, &opts(true, None), true, &mut buf);
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("\x1b["));
        assert!(out.contains("Root"));
        assert!(out.contains("Kid"));
    }

    #[test]
    fn no_color_output_has_no_ansi() {
        let data = vec![child("Root", vec![child("Kid", vec![])])];
        let out = render(&data, &opts(true, None));
        assert!(!out.contains("\x1b["));
    }
}
