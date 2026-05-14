use ::html2md;
use reqwest::Client;
use std::error::Error;
use std::time::Duration;

pub struct WebManager {
    client: Client,
}

impl WebManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            client: Client::builder().timeout(Duration::from_secs(30)).build()?,
        })
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

    pub async fn fetch_url(&self, url_str: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let parsed = url::Url::parse(url_str).map_err(|_| format!("Invalid URL: {}", url_str))?;
        let scheme = parsed.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(format!("Blocked URL scheme: {}", scheme).into());
        }
        if let Some(host) = parsed.host_str() {
            // Comprehensive SSRF protection: check all known localhost representations
            if host == "localhost"
                || host == "127.0.0.1"
                || host == "127.1"
                || host == "0.0.0.0"
                || host == "[::1]"
                || host == "::1"
                || host.starts_with("127.")
                || host == "2130706433"
                || host == "0x7f000001"
                || host == "0177.0.0.1"
            {
                return Err(format!("Blocked local host: {}", host).into());
            }
            if host.ends_with(".local") || host.ends_with(".internal") {
                return Err(format!("Blocked private host: {}", host).into());
            }
        }
        let res = self
            .client
            .get(url_str)
            .header("User-Agent", "OpenCrust/0.1.0")
            .send()
            .await?;

        let html = res.text().await?;
        if html.len() > 10_000_000 {
            return Err("Response too large (>10MB)".into());
        }
        Ok(self.html_to_md(&html))
    }

    fn html_to_md(&self, html: &str) -> String {
        // Use html2md crate for proper HTML to Markdown conversion
        html2md::parse_html(html)
    }
}
