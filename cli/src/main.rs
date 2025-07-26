use chrono::{DateTime, NaiveDateTime, Utc};
use clap::Parser;
use colored::*;
use reqwest;
use serde::Deserialize;
use std::error::Error;
use tokio;

#[derive(Parser)]
#[command(name = "recall-search")]
#[command(version = "0.1.0")]
#[command(author = "Your Name")]
#[command(about = "Search through saved web pages with ripgrep-style output")]
struct Args {
    /// Search query string
    query: String,

    /// Maximum number of results to return
    #[arg(short = 'l', long = "limit", default_value = "5")]
    limit: u32,

    /// Server URL
    #[arg(short = 's', long = "server", default_value = "http://localhost:8000")]
    server: String,
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

    // Make request to the API
    let client = reqwest::Client::new();
    let url = format!(
        "{}/search_text?q={}&limit={}",
        args.server,
        urlencoding::encode(&args.query),
        args.limit
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
        println!("No results found for query: {}", args.query.yellow());
        return Ok(());
    }

    // Display results in ripgrep style
    for result in results {
        display_result(&result, &args)?;
        println!(); // Empty line between results
    }

    Ok(())
}

fn display_result(result: &SearchResult, args: &Args) -> Result<(), Box<dyn Error>> {
    // Parse the datetime
    let created_at =
        chrono::NaiveDateTime::parse_from_str(&result.created_at, "%Y-%m-%dT%H:%M:%S%.f")
            .map_err(|e| format!("Failed to parse date '{}': {}", result.created_at, e))?;

    // Display header with title and timestamp
    println!(
        "{} {} {} {}",
        result.title.bright_blue().bold(),
        "•".dimmed(),
        created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed(),
        format!("{}/page/{}", args.server, result.id).bright_purple()
    );

    // Find and display context for each search term
    let search_terms: Vec<&str> = args.query.split_whitespace().collect();
    let content_lower = result.content.to_lowercase();

    // Find all positions where search terms appear
    let mut positions = Vec::new();
    for term in &search_terms {
        let term_lower = term.to_lowercase();
        let mut start = 0;
        while let Some(pos) = content_lower[start..].find(&term_lower) {
            positions.push(start + pos);
            start += pos + 1;
        }
    }

    // Sort positions and remove duplicates
    positions.sort();
    positions.dedup();

    // Group nearby positions to avoid overlapping contexts
    let grouped_positions = group_nearby_positions(positions, 100); // Group if within 100 chars

    for &pos in &grouped_positions {
        display_context(&result.content, pos, &search_terms, 10);
    }

    Ok(())
}

fn group_nearby_positions(positions: Vec<usize>, threshold: usize) -> Vec<usize> {
    if positions.is_empty() {
        return positions;
    }

    let mut grouped = vec![positions[0]];

    for &pos in positions.iter().skip(1) {
        if pos - grouped.last().unwrap() > threshold {
            grouped.push(pos);
        }
    }

    grouped
}

fn display_context(content: &str, match_pos: usize, search_terms: &[&str], context_words: usize) {
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

    // Build context string with highlighting
    let mut context_parts = Vec::new();

    for (i, word) in words[start_idx..end_idx].iter().enumerate() {
        let actual_index = start_idx + i;
        let mut highlighted = false;

        // Check if this word matches any search term
        for term in search_terms {
            if word.to_lowercase().contains(&term.to_lowercase()) {
                context_parts.push(word.red().bold().to_string());
                highlighted = true;
                break;
            }
        }

        if !highlighted {
            context_parts.push(word.to_string());
        }
    }

    let context = context_parts.join(" ");

    // Add ellipsis if we're not at the beginning/end
    let prefix = if start_idx > 0 { "..." } else { "" };
    let suffix = if end_idx < words.len() { "..." } else { "" };

    println!("  {}{}{}", prefix.dimmed(), context, suffix.dimmed());
}
