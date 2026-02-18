mod lib {
    pub mod api;
    pub mod client;
    pub mod types;
}
pub mod cli;

#[tokio::main]
async fn main() {
    if let Err(e) = cli::execute().await {
        let msg = e.to_string();

        // Classify the error for the exit code.
        let code = if msg.contains("Connection refused")
            || msg.contains("tcp connect error")
            || msg.contains("connection error")
        {
            eprintln!(
                "Error: Eagle server not reachable at localhost:{}",
                cli::PORT
            );
            cli::output::exit_code::CONNECTION
        } else {
            eprintln!("Error: {}", e);
            cli::output::exit_code::ERROR
        };

        std::process::exit(code);
    }
}
