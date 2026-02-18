use clap::{Arg, ArgMatches, Command};
use clap_complete::{generate, Shell};
use std::io;

/// Build the `completions` subcommand (hidden from help).
pub fn build() -> Command {
    Command::new("completions")
        .about("Generate shell completion scripts")
        .hide(true)
        .arg(
            Arg::new("shell")
                .long("shell")
                .value_name("SHELL")
                .help("Target shell: bash, zsh, fish, powershell, elvish")
                .required(true)
                .value_parser(["bash", "zsh", "fish", "powershell", "elvish"]),
        )
}

/// Generate shell completions and write to stdout.
pub fn execute(matches: &ArgMatches) {
    let shell_name = matches
        .get_one::<String>("shell")
        .expect("shell is required");

    let shell: Shell = match shell_name.as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        _ => {
            eprintln!(
                "Error: unknown shell '{}'. Supported: bash, zsh, fish, powershell, elvish",
                shell_name
            );
            std::process::exit(super::output::exit_code::USAGE);
        }
    };

    let mut cmd = super::build_command();
    generate(shell, &mut cmd, "eagle-eye", &mut io::stdout());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completions_command_is_hidden() {
        let cmd = build();
        assert!(cmd.is_hide_set());
    }

    #[test]
    fn completions_bash_generates_output() {
        let mut cmd = super::super::build_command();
        let mut buf = Vec::new();
        generate(Shell::Bash, &mut cmd, "eagle-eye", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("eagle-eye"));
        assert!(!output.is_empty());
    }

    #[test]
    fn completions_zsh_generates_output() {
        let mut cmd = super::super::build_command();
        let mut buf = Vec::new();
        generate(Shell::Zsh, &mut cmd, "eagle-eye", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("eagle-eye"));
    }

    #[test]
    fn completions_fish_generates_output() {
        let mut cmd = super::super::build_command();
        let mut buf = Vec::new();
        generate(Shell::Fish, &mut cmd, "eagle-eye", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("eagle-eye"));
    }
}
