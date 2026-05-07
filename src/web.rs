use ::html2md;
use reqwest::Client;
use std::error::Error;

pub struct WebManager {
    client: Client,
}

impl WebManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn search(&self, query: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        // Placeholder for Brave/Google Search API
        // For now, we'll return a message that API keys are required
        Ok(format!(
            "Searching for: '{}'. (Note: Set BRAVE_API_KEY in config to enable real search)",
            query
        ))
    }

    pub async fn fetch_url(&self, url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let res = self
            .client
            .get(url)
            .header("User-Agent", "OpenCrust/0.1.0")
            .send()
            .await?;

        let html = res.text().await?;
        Ok(self.html_to_md(&html))
    }

    fn html_to_md(&self, html: &str) -> String {
        // Use html2md crate for proper HTML to Markdown conversion
        html2md::parse_html(html)
    }
}
