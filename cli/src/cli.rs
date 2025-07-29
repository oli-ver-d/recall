use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "recall")]
#[command(version = "0.1.0")]
#[command(author = "Oliver Dennis")]
#[command(about = "Archive and search web pages")]
pub struct Args {
    /// Server URL
    #[arg(
        short = 's',
        long = "server",
        default_value = "http://localhost:8000",
        env = "RECALL_SERVER"
    )]
    pub server: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Search through saved web pages
    Search {
        /// Search query string
        query: String,
        /// Maximum number of results to return
        #[arg(short = 'l', long = "limit", default_value = "5")]
        limit: u32,
        /// Tags to filter the search by
        #[arg(short = 't', long = "tags")]
        tags: Vec<String>,
        /// Set to only show where the query matches a whole word
        #[arg(short = 'w', long = "whole")]
        whole: bool,
    },
    /// Save a URL to the archive
    Save {
        /// URL to save
        url: String,
        /// Tags to add to the saved page
        #[arg(short = 't', long = "tags")]
        tags: Vec<String>,
    },
    /// Open a saved recall item in the browser
    Open {
        /// Id of the saved item to open
        id: u32,
        /// Whether to open the original url, rather than the saved version
        #[arg(short = 'o', long = "original")]
        original: bool,
    },
}
