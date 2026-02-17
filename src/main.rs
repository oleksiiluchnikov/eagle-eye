mod lib {
    pub mod api;
    pub mod client;
    pub mod types;
}
pub mod cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::execute().await?;
    Ok(())
}
