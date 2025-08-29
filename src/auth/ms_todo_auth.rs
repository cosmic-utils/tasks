use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use tracing::info;
use rand::Rng;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Response, Server};
use url::Url;

use crate::auth::token_store::{AuthConfig, TokenStore};

/// Azure application client ID (reused from reference)
const CLIENT_ID: &str = "95367b4f-624c-452c-b099-bfc9c27b69b9";

/// OAuth redirect URI
const REDIRECT_URI: &str = "http://localhost:8080/callback";

/// OAuth scopes for Microsoft Todo access
const SCOPES: &str = "https://graph.microsoft.com/User.Read https://graph.microsoft.com/Tasks.ReadWrite https://graph.microsoft.com/Tasks.ReadWrite.Shared openid profile email offline_access";

/// Microsoft OAuth authorization URL
const AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";

/// Microsoft OAuth token URL
const TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";

/// Local server port for OAuth callback
const CALLBACK_PORT: u16 = 8080;

/// Local server address
const CALLBACK_ADDRESS: &str = "127.0.0.1";

/// Token refresh buffer time in seconds (refresh 5 minutes before expiry)
const TOKEN_REFRESH_BUFFER_SECS: u64 = 300;

/// PKCE code verifier length
const PKCE_CODE_VERIFIER_LENGTH: usize = 128;

/// PKCE character set
const PKCE_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

/// HTML response for successful authorization
const SUCCESS_HTML: &str = include_str!("./success.html");

/// Token response from Microsoft OAuth
#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    token_type: String,
}

/// Microsoft Todo authentication manager
#[derive(Debug, Clone)]
pub struct MsTodoAuth {
    client: Client,
    token_store: TokenStore,
}

impl MsTodoAuth {
    /// Create a new authentication manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            token_store: TokenStore::new()?,
        })
    }

    /// Generate PKCE code verifier and challenge
    fn generate_pkce() -> (String, String) {
        let code_verifier: String = (0..PKCE_CODE_VERIFIER_LENGTH)
            .map(|_| PKCE_CHARS[rand::rng().random_range(0..PKCE_CHARS.len())] as char)
            .collect();

        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

        (code_verifier, code_challenge)
    }

    /// Start the OAuth authorization flow
    pub fn authorize(&self) -> Result<AuthConfig> {
        let (code_verifier, code_challenge) = Self::generate_pkce();

        // Build authorization URL
        let auth_url = self.build_auth_url(&code_challenge)?;

        info!("🌐 Opening browser for Microsoft Todo authentication...");
        info!("🔗 Auth URL: {}", auth_url.as_str());
        webbrowser::open(auth_url.as_str()).context("Failed to open browser")?;

        // Start local server to receive callback
        let server = Server::http(format!("{}:{}", CALLBACK_ADDRESS, CALLBACK_PORT))
            .map_err(|e| anyhow!("Failed to start local server: {}", e))?;

        info!("⏳ Waiting for authorization callback...");

        self.handle_authorization_callback(server, &code_verifier)
    }

    /// Build the authorization URL with all required parameters
    fn build_auth_url(&self, code_challenge: &str) -> Result<Url> {
        let mut auth_url = Url::parse(AUTH_URL)?;
        auth_url
            .query_pairs_mut()
            .append_pair("client_id", CLIENT_ID)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", REDIRECT_URI)
            .append_pair("scope", SCOPES)
            .append_pair("code_challenge", code_challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("response_mode", "query")
            .append_pair("prompt", "consent");

        Ok(auth_url)
    }

    /// Handle the authorization callback from the browser
    fn handle_authorization_callback(
        &self,
        server: Server,
        code_verifier: &str,
    ) -> Result<AuthConfig> {
        for request in server.incoming_requests() {
            let url = format!("http://localhost:{}", CALLBACK_PORT);
            let full_url = format!("{}{}", url, request.url());
            let parsed_url = Url::parse(&full_url)?;

            if let Some(code) = self.extract_authorization_code(&parsed_url) {
                // Send success response to browser
                self.send_success_response(request)?;

                // Exchange code for tokens
                let tokens = self.exchange_code_for_tokens(&code, code_verifier)?;
                let config = self.save_tokens(tokens)?;
                return Ok(config);
            }

            if self.is_authorization_error(&parsed_url) {
                self.send_error_response(request)?;
                return Err(anyhow!("Authorization was denied or failed"));
            }
        }

        Err(anyhow!("Authorization flow incomplete"))
    }

    /// Extract authorization code from URL
    fn extract_authorization_code(&self, url: &Url) -> Option<String> {
        url.query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value.to_string())
    }

    /// Check if URL contains authorization error
    fn is_authorization_error(&self, url: &Url) -> bool {
        url.query_pairs().any(|(key, _)| key == "error")
    }

    /// Send success response to browser
    fn send_success_response(&self, request: tiny_http::Request) -> Result<()> {
        let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..])
            .map_err(|_| anyhow!("Failed to create content-type header"))?;
        let response = Response::from_string(SUCCESS_HTML).with_header(header);
        request
            .respond(response)
            .context("Failed to send success response")?;
        Ok(())
    }

    /// Send error response to browser
    fn send_error_response(&self, request: tiny_http::Request) -> Result<()> {
        let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..])
            .map_err(|_| anyhow!("Failed to create content-type header"))?;
        let response = Response::from_string("Authorization failed!").with_header(header);
        request
            .respond(response)
            .context("Failed to send error response")?;
        Ok(())
    }

    /// Exchange authorization code for tokens
    fn exchange_code_for_tokens(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<TokenResponse> {
        let params = self.build_token_exchange_params(code, code_verifier);

        let response = self
            .client
            .post(TOKEN_URL)
            .form(&params)
            .send()
            .context("Failed to send token exchange request")?;

        if !response.status().is_success() {
            let error_text = response.text()?;
            return Err(anyhow!("Token exchange failed: {}", error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .context("Failed to parse token response")?;
        Ok(token_response)
    }

    /// Build parameters for token exchange
    fn build_token_exchange_params(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> HashMap<&'static str, String> {
        let mut params = HashMap::new();
        params.insert("client_id", CLIENT_ID.to_string());
        params.insert("code", code.to_string());
        params.insert("redirect_uri", REDIRECT_URI.to_string());
        params.insert("grant_type", "authorization_code".to_string());
        params.insert("code_verifier", code_verifier.to_string());
        params
    }

    /// Save tokens to storage
    fn save_tokens(&self, tokens: TokenResponse) -> Result<AuthConfig> {
        let expires_at =
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + tokens.expires_in;

        let config = AuthConfig {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token.unwrap_or_default(),
            expires_at,
        };

        self.token_store
            .save_tokens(&config)
            .context("Failed to save tokens")?;

        info!(
            "✅ Microsoft Todo tokens saved successfully using: {}",
            self.token_store.get_storage_info()
        );
        Ok(config)
    }

    /// Load tokens from storage
    pub fn load_tokens(&self) -> Result<AuthConfig> {
        self.token_store
            .load_tokens()
            .context("Failed to load tokens from storage")
    }

    /// Check if token is expired
    pub fn is_token_expired(&self, config: &AuthConfig) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= config.expires_at - TOKEN_REFRESH_BUFFER_SECS
    }

    /// Check if we have valid tokens
    pub fn has_valid_tokens(&self) -> bool {
        match self.load_tokens() {
            Ok(config) => !self.is_token_expired(&config),
            Err(_) => false,
        }
    }

    /// Get valid access token with automatic refresh if needed
    pub fn get_access_token(&self) -> Result<String> {
        let mut config = self.load_tokens()?;
        
        if self.is_token_expired(&config) {
            info!("🔄 Token expired, attempting to refresh...");
            config = self.refresh_tokens(&config)?;
        }
        
        Ok(config.access_token)
    }

    /// Refresh access token using refresh token
    pub fn refresh_tokens(&self, config: &AuthConfig) -> Result<AuthConfig> {
        if config.refresh_token.is_empty() {
            return Err(anyhow!("No refresh token available"));
        }

        let params = self.build_refresh_token_params(&config.refresh_token);

        let response = self
            .client
            .post(TOKEN_URL)
            .form(&params)
            .send()
            .context("Failed to send token refresh request")?;

        if !response.status().is_success() {
            let error_text = response.text()?;
            return Err(anyhow!("Token refresh failed: {}", error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .context("Failed to parse token refresh response")?;

        // Save the new tokens
        let new_config = self.save_tokens(token_response)?;
        
        info!("✅ Access token refreshed successfully");
        Ok(new_config)
    }

    /// Build parameters for token refresh
    fn build_refresh_token_params(&self, refresh_token: &str) -> HashMap<&'static str, String> {
        let mut params = HashMap::new();
        params.insert("client_id", CLIENT_ID.to_string());
        params.insert("refresh_token", refresh_token.to_string());
        params.insert("grant_type", "refresh_token".to_string());
        params
    }
}
