//! Authentication module — GitHub Copilot and ChatGPT Plus OAuth login
//!
//! Implements device flow OAuth for GitHub Copilot and ChatGPT Plus,
//! allowing users to authenticate with their existing subscriptions
//! and use them as LLM providers in OpenCrust.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Cached OAuth token with expiry information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedToken {
    pub token: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub provider: AuthProvider,
}

/// Supported auth providers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthProvider {
    GitHubCopilot,
    ChatGptPlus,
}

impl std::fmt::Display for AuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthProvider::GitHubCopilot => write!(f, "GitHub Copilot"),
            AuthProvider::ChatGptPlus => write!(f, "ChatGPT Plus"),
        }
    }
}

/// Device flow response from OAuth provider.
#[derive(Debug, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Token exchange response.
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_in: Option<u64>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Get the auth directory path.
fn auth_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/opencrust/auth")
}

/// Get the token cache file path for a provider.
fn token_cache_path(provider: &AuthProvider) -> PathBuf {
    let filename = match provider {
        AuthProvider::GitHubCopilot => "github_copilot.json",
        AuthProvider::ChatGptPlus => "chatgpt_plus.json",
    };
    auth_dir().join(filename)
}

/// Save a cached token to disk.
pub fn save_token(token: &CachedToken) -> std::io::Result<()> {
    let dir = auth_dir();
    std::fs::create_dir_all(&dir)?;
    let path = token_cache_path(&token.provider);
    let content = serde_json::to_string(&token).map_err(std::io::Error::other)?;
    std::fs::write(path, content)
}

/// Load a cached token from disk, returning None if not found or expired.
pub fn load_token(provider: &AuthProvider) -> Option<CachedToken> {
    let path = token_cache_path(provider);
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    let token: CachedToken = serde_json::from_str(&content).ok()?;

    // Check expiry
    if let Some(expires_at) = token.expires_at {
        if chrono::Utc::now() >= expires_at {
            let _ = std::fs::remove_file(path);
            return None;
        }
    }

    Some(token)
}

/// Clear cached token for a provider.
pub fn clear_token(provider: &AuthProvider) -> std::io::Result<()> {
    let path = token_cache_path(provider);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Initiate GitHub Copilot device flow authentication.
/// Returns the user code and verification URL for the user to visit.
pub async fn github_copilot_device_flow() -> Result<DeviceCodeResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://github.com/login/device/code")
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .body(
            serde_json::json!({
                "client_id": "Iv1.b507a08c67ecfe98",
                "scope": "read:user"
            })
            .to_string(),
        )
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    resp.json::<DeviceCodeResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Poll for GitHub Copilot token after device code authorization.
pub async fn github_copilot_poll_token(device_code: &str) -> Result<CachedToken, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://github.com/login/oauth/access_token")
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .body(
            serde_json::json!({
                "client_id": "Iv1.b507a08c67ecfe98",
                "device_code": device_code,
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code"
            })
            .to_string(),
        )
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    if let Some(error) = token_resp.error {
        return Err(format!(
            "Auth error: {} - {}",
            error,
            token_resp.error_description.unwrap_or_default()
        ));
    }

    let access_token = token_resp.access_token.ok_or("No access token returned")?;
    let expires_at = token_resp
        .expires_in
        .map(|e| chrono::Utc::now() + chrono::Duration::seconds(e as i64));

    let cached = CachedToken {
        token: access_token,
        expires_at,
        provider: AuthProvider::GitHubCopilot,
    };

    let _ = save_token(&cached);
    Ok(cached)
}

/// Initiate ChatGPT Plus device flow authentication.
/// Returns a user code and verification URL.
pub async fn chatgpt_plus_device_flow() -> Result<DeviceCodeResponse, String> {
    // ChatGPT Plus uses a simulated device flow for local auth
    // In production this would hit OpenAI's device auth endpoint
    let client = reqwest::Client::new();
    let resp = client
        .post("https://auth0.openai.com/oauth/device/code")
        .header("content-type", "application/json")
        .body(
            serde_json::json!({
                "client_id": "DRivsnm2Mu42T3KOpqdtwB3NYviHYzwD",
                "scope": "openid profile email",
                "audience": "https://api.openai.com/v1"
            })
            .to_string(),
        )
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    resp.json::<DeviceCodeResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Poll for ChatGPT Plus token after device code authorization.
pub async fn chatgpt_plus_poll_token(device_code: &str) -> Result<CachedToken, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://auth0.openai.com/oauth/token")
        .header("content-type", "application/json")
        .body(
            serde_json::json!({
                "client_id": "DRivsnm2Mu42T3KOpqdtwB3NYviHYzwD",
                "device_code": device_code,
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code"
            })
            .to_string(),
        )
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    if let Some(error) = token_resp.error {
        return Err(format!(
            "Auth error: {} - {}",
            error,
            token_resp.error_description.unwrap_or_default()
        ));
    }

    let access_token = token_resp.access_token.ok_or("No access token returned")?;
    let expires_at = token_resp
        .expires_in
        .map(|e| chrono::Utc::now() + chrono::Duration::seconds(e as i64));

    let cached = CachedToken {
        token: access_token,
        expires_at,
        provider: AuthProvider::ChatGptPlus,
    };

    let _ = save_token(&cached);
    Ok(cached)
}

/// Check if a provider has a valid (non-expired) cached token.
pub fn is_authenticated(provider: &AuthProvider) -> bool {
    load_token(provider).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_dir_is_deterministic() {
        let dir1 = auth_dir();
        let dir2 = auth_dir();
        assert_eq!(dir1, dir2);
    }

    #[test]
    fn token_cache_path_varies_by_provider() {
        let p1 = token_cache_path(&AuthProvider::GitHubCopilot);
        let p2 = token_cache_path(&AuthProvider::ChatGptPlus);
        assert_ne!(p1, p2);
    }

    #[test]
    fn save_and_load_token_roundtrip() {
        let token = CachedToken {
            token: "test_token_abc".to_string(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            provider: AuthProvider::GitHubCopilot,
        };
        let _ = save_token(&token);
        let loaded = load_token(&AuthProvider::GitHubCopilot);
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().token, "test_token_abc");
        // Cleanup
        let _ = clear_token(&AuthProvider::GitHubCopilot);
    }

    #[test]
    fn clear_token_removes_file() {
        let token = CachedToken {
            token: "to_clear".to_string(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            provider: AuthProvider::ChatGptPlus,
        };
        let _ = save_token(&token);
        assert!(load_token(&AuthProvider::ChatGptPlus).is_some());
        let _ = clear_token(&AuthProvider::ChatGptPlus);
        assert!(load_token(&AuthProvider::ChatGptPlus).is_none());
    }

    #[test]
    fn expired_token_returns_none() {
        let token = CachedToken {
            token: "expired".to_string(),
            expires_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
            provider: AuthProvider::GitHubCopilot,
        };
        let _ = save_token(&token);
        // Should return None because token is expired
        let loaded = load_token(&AuthProvider::GitHubCopilot);
        assert!(loaded.is_none());
        // Cleanup
        let _ = clear_token(&AuthProvider::GitHubCopilot);
    }
}
