use clap::Parser;
use std::error::Error;
use tokio;

mod api;
mod cli;
mod commands;
mod display;

use cli::Args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    commands::execute(args).await
}
