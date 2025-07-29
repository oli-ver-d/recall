use crate::api::SearchResult;
use chrono::NaiveDateTime;
use colored::*;
use std::error::Error;

pub fn display_search_results(
    results: &[SearchResult],
    query: &str,
    server: &str,
    tags: &[String],
    whole: bool,
) -> Result<(), Box<dyn Error>> {
    if results.is_empty() {
        display_no_results(query, tags);
        return Ok(());
    }

    // Display results in ripgrep style
    for result in results.iter().rev() {
        display_result(result, query, server, whole)?;
        println!(); // Empty line between results
    }

    Ok(())
}

pub fn display_no_results(query: &str, tags: &[String]) {
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
}

pub fn display_save_success(url: &str, tags: &[String], id: i32) {
    println!("Saving URL: {}", url.bright_blue());
    if !tags.is_empty() {
        println!("Tags: {}", tags.join(", ").dimmed());
    }
    println!(
        "{} Saved successfully with ID: {}",
        "✓".green().bold(),
        id.to_string().bright_green()
    );
}

pub fn display_save_error(error: &str) {
    eprintln!("{} Failed to save URL: {}", "✗".red().bold(), error);
}

pub fn display_open_success(url: &str) {
    println!("Opened browser to {}", url);
}

pub fn display_open_error() {
    eprintln!("Failed to open browser");
}

fn display_result(
    result: &SearchResult,
    query: &str,
    server: &str,
    whole: bool,
) -> Result<(), Box<dyn Error>> {
    // Parse the datetime - handle FastAPI ISO format
    let created_at = parse_datetime(&result.created_at)?;

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
    display_search_matches(&result.content, query, whole);

    Ok(())
}

fn parse_datetime(datetime_str: &str) -> Result<NaiveDateTime, Box<dyn Error>> {
    NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S%.f"))
        .map_err(|e| format!("Failed to parse date '{}': {}", datetime_str, e).into())
}

fn display_search_matches(content: &str, query: &str, whole: bool) {
    let content_lower = content.to_lowercase();
    let query_lower = query.to_lowercase();

    // Find all positions where the exact phrase appears
    let positions = find_match_positions(&content_lower, &query_lower, whole);

    // Display context for the first 5 matches only
    for &pos in positions.iter().take(5) {
        display_context(content, pos, query, 10);
    }
}

fn find_match_positions(content_lower: &str, query_lower: &str, whole: bool) -> Vec<usize> {
    let mut positions = Vec::new();

    if whole {
        let regex_pattern = format!(r"\b{}\b", regex::escape(query_lower));
        if let Ok(re) = regex::Regex::new(&regex_pattern) {
            for mat in re.find_iter(content_lower) {
                positions.push(mat.start());
            }
        }
    } else {
        let mut start = 0;
        while let Some(pos) = content_lower[start..].find(query_lower) {
            positions.push(start + pos);
            start += pos + 1;
        }
    }

    positions
}

fn display_context(content: &str, match_pos: usize, query: &str, context_words: usize) {
    let words: Vec<&str> = content.split_whitespace().collect();

    // Find the word index containing the match position
    let word_index = find_word_index(&words, match_pos);

    // Calculate context boundaries
    let start_idx = word_index.saturating_sub(context_words);
    let end_idx = std::cmp::min(words.len(), word_index + context_words + 1);

    // Build context string with highlighting for the exact phrase
    let context_text = words[start_idx..end_idx].join(" ");
    let highlighted_context = highlight_query_in_context(&context_text, query);

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

fn find_word_index(words: &[&str], match_pos: usize) -> usize {
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

    word_index
}

fn highlight_query_in_context(context_text: &str, query: &str) -> String {
    let query_lower = query.to_lowercase();
    let context_lower = context_text.to_lowercase();

    // Find and highlight the exact phrase in the context
    if let Some(phrase_pos) = context_lower.find(&query_lower) {
        let before = &context_text[..phrase_pos];
        let matched = &context_text[phrase_pos..phrase_pos + query.len()];
        let after = &context_text[phrase_pos + query.len()..];
        format!("{}{}{}", before, matched.red().bold(), after)
    } else {
        context_text.to_string()
    }
}
