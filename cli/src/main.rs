use chrono::{DateTime, NaiveDateTime, Utc};
use clap::{Parser, Subcommand};
use colored::*;
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio;

#[derive(Parser)]
#[command(name = "recall")]
#[command(version = "0.1.0")]
#[command(author = "Your Name")]
#[command(about = "Archive and search web pages")]
struct Args {
    /// Server URL
    #[arg(short = 's', long = "server", default_value = "http://localhost:8000")]
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
    },
    /// Save a URL to the archive
    Save {
        /// URL to save
        url: String,
        /// Tags to add to the saved page
        #[arg(short = 't', long = "tags")]
        tags: Vec<String>,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Search { query, limit } => {
            search_pages(&args.server, &query, limit).await?;
        }
        Commands::Save { url, tags } => {
            save_page(&args.server, &url, tags).await?;
        }
    }

    Ok(())
}

async fn search_pages(server: &str, query: &str, limit: u32) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/search_text?q={}&limit={}",
        server,
        urlencoding::encode(query),
        limit
    );

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
        println!("No results found for query: {}", query.yellow());
        return Ok(());
    }

    // Display results in ripgrep style
    for result in results {
        display_result(&result, query, server)?;
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

fn display_result(result: &SearchResult, query: &str, server: &str) -> Result<(), Box<dyn Error>> {
    // Parse the datetime - handle FastAPI ISO format
    let created_at =
        chrono::NaiveDateTime::parse_from_str(&result.created_at, "%Y-%m-%dT%H:%M:%S%.f")
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&result.created_at, "%Y-%m-%d %H:%M:%S%.f")
            })
            .map_err(|e| format!("Failed to parse date '{}': {}", result.created_at, e))?;

    // Display header with title and timestamp
    println!(
        "{} {} {} {} {}",
        result.title.bright_blue().bold(),
        "•",
        format!("{}/page/{}", server, result.id)
            .bright_purple()
            .bold(),
        "•",
        created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed(),
    );

    // Find and display context for the exact phrase
    let content_lower = result.content.to_lowercase();
    let query_lower = query.to_lowercase();

    // Find all positions where the exact phrase appears
    let mut positions = Vec::new();
    let mut start = 0;
    while let Some(pos) = content_lower[start..].find(&query_lower) {
        positions.push(start + pos);
        start += pos + 1;
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
