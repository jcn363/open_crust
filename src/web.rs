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
        let api_key = std::env::var("BRAVE_API_KEY").map_err(|_| {
            "BRAVE_API_KEY environment variable not set. Set it to enable Brave Search.".to_string()
        })?;

        let response = self
            .client
            .post("https://api.search.brave.com/res/v1/web/search")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "q": query,
                "count": 5
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Brave Search API error: {}", response.status()).into());
        }

        let json: serde_json::Value = response.json().await?;
        let results: Vec<_> = json
            .get("web")
            .and_then(|w| w.get("results"))
            .and_then(|r| r.as_array())
            .map(|a| a.to_vec())
            .unwrap_or_default();

        let mut output = format!("Search results for '{}':\n\n", query);
        for (i, result) in results.iter().enumerate() {
            let title = result
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("No title");
            let url = result.get("url").and_then(|u| u.as_str()).unwrap_or("");
            let description = result
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            output.push_str(&format!(
                "{}. {}\n   {}\n   {}\n\n",
                i + 1,
                title,
                url,
                description
            ));
        }

        Ok(output)
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
