use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct ApiClient {
    client: reqwest::Client,
    server_url: String,
}

impl ApiClient {
    pub fn new(server_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            server_url,
        }
    }

    pub async fn search(
        &self,
        query: &str,
        limit: u32,
        tags: &[String],
        whole: bool,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let mut url = format!(
            "{}/search_text?q={}&limit={}&whole_word={}",
            self.server_url,
            urlencoding::encode(query),
            limit,
            whole
        );

        for tag in tags {
            url.push_str(&format!("&tags={}", urlencoding::encode(tag)));
        }

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status {}", response.status()).into());
        }

        let results: Vec<SearchResult> = response.json().await?;
        Ok(results)
    }

    pub async fn save(&self, url: &str, tags: Vec<String>) -> Result<SaveResponse, Box<dyn Error>> {
        let save_url = format!("{}/save/", self.server_url);
        let request = SaveRequest {
            url: url.to_string(),
            tags,
        };

        let response = self.client.post(&save_url).json(&request).send().await?;

        if response.status().is_success() {
            let save_response: SaveResponse = response.json().await?;
            Ok(save_response)
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to save URL: {}", error_text).into())
        }
    }

    pub async fn get_original_url(&self, id: u32) -> Result<String, Box<dyn Error>> {
        let get_url = format!("{}/get_url?id={}", self.server_url, id);
        let response = self.client.get(&get_url).send().await?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status {}", response.status()).into());
        }

        let result: GetUrlResult = response.json().await?;
        Ok(result.url)
    }

    pub fn get_page_url(&self, id: u32) -> String {
        format!("{}/page/{}", self.server_url, id)
    }
}

#[derive(Serialize)]
pub struct SaveRequest {
    pub url: String,
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct SaveResponse {
    pub status: String,
    pub id: i32,
}

#[derive(Deserialize)]
pub struct SearchResult {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub tags: String,
    pub content: String,
    pub saved_path: String,
    pub created_at: String,
}

#[derive(Deserialize)]
struct GetUrlResult {
    pub url: String,
}
