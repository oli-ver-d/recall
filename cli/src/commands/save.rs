use crate::{api::ApiClient, display};
use std::error::Error;

pub async fn handle_save(
    client: &ApiClient,
    url: &str,
    tags: Vec<String>,
) -> Result<(), Box<dyn Error>> {
    match client.save(url, tags.clone()).await {
        Ok(response) => {
            display::display_save_success(url, &tags, response.id);
        }
        Err(e) => {
            display::display_save_error(&e.to_string());
        }
    }
    Ok(())
}
