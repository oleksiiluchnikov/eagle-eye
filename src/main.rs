mod lib {
    pub mod api;
    pub mod client;
    pub mod types;
}
pub mod cli;

#[tokio::main]
async fn main() {
    let matches = cli::get_matches();
    let json_mode = cli::is_json_mode(&matches);

    if let Err(e) = cli::execute_with_matches(&matches).await {
        let msg = e.to_string();

        // Classify the error for the exit code.
        let code = if msg.contains("Connection refused")
            || msg.contains("tcp connect error")
            || msg.contains("connection error")
        {
            cli::output::output_error(
                &format!("Eagle server not reachable at localhost:{}", cli::PORT),
                json_mode,
            );
            cli::output::exit_code::CONNECTION
        } else {
            cli::output::output_error(&msg, json_mode);
            cli::output::exit_code::ERROR
        };

        std::process::exit(code);
    }
}
