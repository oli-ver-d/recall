use crate::{api::ApiClient, display};
use std::error::Error;

pub async fn handle_search(
    client: &ApiClient,
    server: &str,
    query: &str,
    limit: u32,
    tags: &[String],
    whole: bool,
) -> Result<(), Box<dyn Error>> {
    let results = client.search(query, limit, tags, whole).await?;
    display::display_search_results(&results, query, server, tags, whole)?;
    Ok(())
}
