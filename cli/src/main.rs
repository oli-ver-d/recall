use clap::{Parser, Subcommand};
use colored::*;
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio;

#[derive(Parser)]
#[command(name = "recall")]
#[command(version = "0.1.0")]
#[command(author = "Oliver Dennis")]
#[command(about = "Archive and search web pages")]
struct Args {
    /// Server URL
    #[arg(
        short = 's',
        long = "server",
        default_value = "http://localhost:8000",
        env = "RECALL_SERVER"
    )]
    server: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

#[derive(Serialize)]
struct SaveRequest {
    url: String,
    tags: Vec<String>,
}

#[derive(Deserialize)]
struct SaveResponse {
    status: String,
    id: i32,
}

#[derive(Deserialize)]
struct SearchResult {
    id: i32,
    url: String,
    title: String,
    tags: String,
    content: String,
    saved_path: String,
    created_at: String,
}

#[derive(Deserialize)]
struct GetUrlResult {
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Search {
            query,
            limit,
            tags,
            whole,
        } => {
            search_pages(&args.server, &query, limit, &tags, whole).await?;
        }
        Commands::Save { url, tags } => {
            save_page(&args.server, &url, tags).await?;
        }
        Commands::Open { id, original } => {
            open_page(&args.server, id, original).await?;
        }
    }

    Ok(())
}

async fn search_pages(
    server: &str,
    query: &str,
    limit: u32,
    tags: &Vec<String>,
    whole: bool,
) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let mut url = format!(
        "{}/search_text?q={}&limit={}&whole_word={}",
        server,
        urlencoding::encode(query),
        limit,
        whole
    );

    for tag in tags {
        url.push_str(&format!("&tags={}", urlencoding::encode(tag)));
    }

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        eprintln!(
            "Error: API request failed with status {}",
            response.status()
        );
        return Ok(());
    }

    let results: Vec<SearchResult> = response.json().await?;

    if results.is_empty() {
        let tag_info = if tags.is_empty() {
            String::new()
        } else {
            format!(" with tags [{}]", tags.join(", "))
        };
        println!(
            "No results found for query: {}{}",
            query.yellow(),
            tag_info.dimmed()
        );
        return Ok(());
    }

    // Display results in ripgrep style
    for result in results.iter().rev() {
        display_result(&result, query, server, whole)?;
        println!(); // Empty line between results
    }

    Ok(())
}

async fn save_page(server: &str, url: &str, tags: Vec<String>) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let save_url = format!("{}/save/", server);

    let request = SaveRequest {
        url: url.to_string(),
        tags,
    };

    println!("Saving URL: {}", url.bright_blue());
    if !request.tags.is_empty() {
        println!("Tags: {}", request.tags.join(", ").dimmed());
    }

    let response = client.post(&save_url).json(&request).send().await?;

    if response.status().is_success() {
        let save_response: SaveResponse = response.json().await?;
        println!(
            "{} Saved successfully with ID: {}",
            "✓".green().bold(),
            save_response.id.to_string().bright_green()
        );
    } else {
        let error_text = response.text().await?;
        eprintln!("{} Failed to save URL: {}", "✗".red().bold(), error_text);
    }

    Ok(())
}

async fn open_page(server: &str, id: u32, original: bool) -> Result<(), Box<dyn Error>> {
    let url = if original {
        let client = reqwest::Client::new();
        let get_url = format!("{}/get_url?id={}", server, id.to_string());

        let response = client.get(&get_url).send().await?;

        if !response.status().is_success() {
            eprintln!(
                "Error: API request failed with status {}",
                response.status()
            );
            return Ok(());
        }

        let result: GetUrlResult = response.json().await?;
        result.url
    } else {
        format!("{}/page/{}", server, id)
    };
    if webbrowser::open(&url).is_ok() {
        println!("Opened browser to {}", url);
    } else {
        eprintln!("Failed to open browser");
    }
    Ok(())
}

fn display_result(
    result: &SearchResult,
    query: &str,
    server: &str,
    whole: bool,
) -> Result<(), Box<dyn Error>> {
    // Parse the datetime - handle FastAPI ISO format
    let created_at =
        chrono::NaiveDateTime::parse_from_str(&result.created_at, "%Y-%m-%dT%H:%M:%S%.f")
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&result.created_at, "%Y-%m-%d %H:%M:%S%.f")
            })
            .map_err(|e| format!("Failed to parse date '{}': {}", result.created_at, e))?;

    // Display header with title and timestamp
    println!(
        "{}",
        created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed()
    );
    println!(
        "ID: {}, {}",
        result.id.to_string().bright_cyan(),
        result.url.bright_green().bold()
    );
    println!(
        "{} {} {}",
        result.title.bright_blue().bold(),
        "•",
        format!("{}/page/{}", server, result.id)
            .bright_purple()
            .bold(),
    );

    // Find and display context for the exact phrase
    let content_lower = result.content.to_lowercase();
    let query_lower = query.to_lowercase();

    // Find all positions where the exact phrase appears
    let mut positions = Vec::new();

    if whole {
        let regex_pattern = format!(r"\b{}\b", regex::escape(&query_lower));
        if let Ok(re) = regex::Regex::new(&regex_pattern) {
            for mat in re.find_iter(&content_lower) {
                positions.push(mat.start());
            }
        }
    } else {
        let mut start = 0;
        while let Some(pos) = content_lower[start..].find(&query_lower) {
            positions.push(start + pos);
            start += pos + 1;
        }
    }

    // Display context for the first 5 matches only
    for &pos in positions.iter().take(5) {
        display_context(&result.content, pos, query, 10);
    }

    Ok(())
}

fn display_context(content: &str, match_pos: usize, query: &str, context_words: usize) {
    let words: Vec<&str> = content.split_whitespace().collect();

    // Find the word index containing the match position
    let mut char_count = 0;
    let mut word_index = 0;

    for (i, word) in words.iter().enumerate() {
        if char_count >= match_pos {
            word_index = i;
            break;
        }
        char_count += word.len() + 1; // +1 for space
        word_index = i + 1;
    }

    // Calculate context boundaries
    let start_idx = word_index.saturating_sub(context_words);
    let end_idx = std::cmp::min(words.len(), word_index + context_words + 1);

    // Build context string with highlighting for the exact phrase
    let context_text = words[start_idx..end_idx].join(" ");
    let query_lower = query.to_lowercase();
    let context_lower = context_text.to_lowercase();

    // Find and highlight the exact phrase in the context
    let highlighted_context = if let Some(phrase_pos) = context_lower.find(&query_lower) {
        let before = &context_text[..phrase_pos];
        let matched = &context_text[phrase_pos..phrase_pos + query.len()];
        let after = &context_text[phrase_pos + query.len()..];
        format!("{}{}{}", before, matched.red().bold(), after)
    } else {
        context_text
    };

    // Add ellipsis if we're not at the beginning/end
    let prefix = if start_idx > 0 { "..." } else { "" };
    let suffix = if end_idx < words.len() { "..." } else { "" };

    println!(
        "  {}{}{}",
        prefix.dimmed(),
        highlighted_context,
        suffix.dimmed()
    );
}
