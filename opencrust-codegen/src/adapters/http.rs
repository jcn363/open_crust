//! HTTP data source adapter.

use crate::adapters::DataSource;
use crate::errors::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

/// HTTP data source that performs a GET request and parses JSON response.
#[allow(dead_code)]
pub struct HttpSource {
    client: Client,
    url: String,
}

impl HttpSource {
    /// Create a new HTTP source with the given URL.
    #[allow(dead_code)]
    pub fn new<U: Into<String>>(url: U) -> Self {
        Self {
            client: Client::new(),
            url: url.into(),
        }
    }

    /// Set a custom `reqwest::Client` (e.g., for auth, timeouts).
    #[allow(dead_code)]
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }
}

#[async_trait]
impl DataSource for HttpSource {
    async fn fetch(&self) -> Result<Value> {
        let resp = self
            .client
            .get(&self.url)
            .send()
            .await?
            .error_for_status()?;
        let json = resp.json::<Value>().await?;
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_http_source_get_json() {
        // Spin up a mock HTTP server.
        let mock_server = MockServer::start().await;
        let body = r#"{ "name": "Bob", "active": true }"#;
        Mock::given(method("GET"))
            .and(path("/data"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&mock_server)
            .await;

        let source = HttpSource::new(format!("{}/data", &mock_server.uri()));
        let result = source.fetch().await.unwrap();
        assert_eq!(result["name"], "Bob");
        assert_eq!(result["active"], true);
    }
}
