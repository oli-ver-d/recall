use crate::{api::ApiClient, display};
use std::error::Error;

pub async fn handle_open(
    client: &ApiClient,
    id: u32,
    original: bool,
) -> Result<(), Box<dyn Error>> {
    let url = if original {
        client.get_original_url(id).await?
    } else {
        client.get_page_url(id)
    };

    if webbrowser::open(&url).is_ok() {
        display::display_open_success(&url);
    } else {
        display::display_open_error();
    }

    Ok(())
}
