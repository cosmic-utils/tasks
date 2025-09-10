use anyhow::{Context, Result};
use reqwest::Client;
use serde::Serialize;
use tracing::debug;

use crate::integration::ms_todo::models::PaginatedCollection;

const GRAPH_API_BASE: &str = "https://graph.microsoft.com/v1.0";

/// HTTP client for Microsoft Graph Todo API operations
#[derive(Debug, Clone)]
pub struct MsTodoHttpClient {
    client: Client,
}

impl MsTodoHttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Get full URL by prepending Graph API base if needed
    pub fn get_full_url(&self, url: &str) -> Result<String> {
        if url.starts_with("http") {
            Ok(url.to_string())
        } else {
            Ok(format!("{}{}", GRAPH_API_BASE, url))
        }
    }

    /// Make a GET request with authorization header
    pub async fn get<T>(&self, url: &str, auth_header: &str) -> Result<T>
    where
        T: Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let url = self.get_full_url(url)?;
        debug!("Getting url: {}", url);

        let http_response = self
            .client
            .get(&url)
            .header("Authorization", auth_header)
            .send()
            .await
            .context("Failed to get response")?;

        // Check if the response is successful
        if !http_response.status().is_success() {
            let status = http_response.status();
            let error_body = http_response.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(anyhow::anyhow!("HTTP {}: {}", status, error_body));
        }

        let response_json = http_response
            .json::<T>()
            .await
            .context("Failed to deserialize response to type T")?;
        Ok(response_json)
    }

    /// Make a POST request with authorization header
    pub async fn post<T, B>(&self, url: &str, body: &B, auth_header: &str) -> Result<T>
    where
        T: Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
        B: Serialize,
    {
        let url = self.get_full_url(url)?;
        debug!("Posting to url: {}", url);

        let http_response = self
            .client
            .post(&url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .context("Failed to get response for post")?;

        // Check if the response is successful
        if !http_response.status().is_success() {
            let status = http_response.status();
            let error_body = http_response.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(anyhow::anyhow!("HTTP {}: {}", status, error_body));
        }

        let response_json = http_response
            .json::<T>()
            .await
            .context("Failed to deserialize response to type T")?;
        Ok(response_json)
    }

    /// Make a DELETE request with authorization header
    pub async fn delete(&self, url: &str, auth_header: &str) -> Result<()> {
        let url = self.get_full_url(url)?;
        debug!("Deleting from url: {}", url);

        let http_response = self
            .client
            .delete(&url)
            .header("Authorization", auth_header)
            .send()
            .await
            .context("Failed to get response for delete")?;

        // Check if the response is successful
        if !http_response.status().is_success() {
            let status = http_response.status();
            let error_body = http_response.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(anyhow::anyhow!("HTTP {}: {}", status, error_body));
        }
        Ok(())
    }

    /// Make a PUT request with authorization header
    #[allow(dead_code)]
    pub async fn put<B, R>(&self, url: &str, body: &B, auth_header: &str) -> Result<R>
    where
        B: Serialize + std::fmt::Debug,
        R: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let url = self.get_full_url(url)?;
        debug!("Putting to url: {}", url);

        let http_response = self
            .client
            .put(&url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .context("Failed to get response for put")?;

        // Check if the response is successful
        if !http_response.status().is_success() {
            let status = http_response.status();
            let error_body = http_response.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(anyhow::anyhow!("HTTP {}: {}", status, error_body));
        }

        let response_json = http_response
            .json::<R>()
            .await
            .context("Failed to deserialize response to type R")?;
        Ok(response_json)
    }

    /// Make a PATCH request with authorization header
    pub async fn patch<B, R>(&self, url: &str, body: &B, auth_header: &str) -> Result<R>
    where
        B: Serialize + std::fmt::Debug,
        R: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let url = self.get_full_url(url)?;
        debug!("Patching url: {}", url);

        let http_response = self
            .client
            .patch(&url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .context("Failed to get response for patch")?;

        // Check if the response is successful
        if !http_response.status().is_success() {
            let status = http_response.status();
            let error_body = http_response.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(anyhow::anyhow!("HTTP {}: {}", status, error_body));
        }

        let response_json = http_response
            .json::<R>()
            .await
            .context("Failed to deserialize response to type R")?;
        Ok(response_json)
    }

    /// Fetch all pages of a paginated collection
    /// This method handles Microsoft Graph API pagination by following next_link URLs
    pub async fn get_all_pages<T, C>(&self, url: &str, auth_header: &str) -> Result<Vec<T>>
    where
        T: Serialize + serde::de::DeserializeOwned + std::fmt::Debug + Clone,
        C: Serialize + serde::de::DeserializeOwned + std::fmt::Debug + PaginatedCollection<T>,
    {
        let mut all_items = Vec::new();
        let mut current_url = url.to_string();
        
        loop {
            debug!("Fetching page from: {}", current_url);
            
            let response: C = self.get(&current_url, auth_header).await?;
            all_items.extend(response.get_items());
            
            // Check if there's a next page
            if let Some(next_link) = response.get_next_link() {
                current_url = next_link;
            } else {
                break;
            }
        }
        
        debug!("Fetched {} total items across all pages", all_items.len());
        Ok(all_items)
    }

    /// Get a request builder for custom HTTP requests
    #[allow(dead_code)]
    pub fn request_builder(&self, method: &str, url: &str) -> reqwest::RequestBuilder {
        match method.to_uppercase().as_str() {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            "PATCH" => self.client.patch(url),
            _ => self.client.get(url), // Default to GET
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_full_url_with_relative_path() {
        let client = MsTodoHttpClient::new();
        let result = client.get_full_url("/me/todo/lists").unwrap();
        assert_eq!(result, "https://graph.microsoft.com/v1.0/me/todo/lists");
    }

    #[test]
    fn test_get_full_url_with_absolute_url() {
        let client = MsTodoHttpClient::new();
        let full_url = "https://example.com/api/test";
        let result = client.get_full_url(full_url).unwrap();
        assert_eq!(result, full_url);
    }

    #[test]
    fn test_get_full_url_with_http_url() {
        let client = MsTodoHttpClient::new();
        let http_url = "http://example.com/api/test";
        let result = client.get_full_url(http_url).unwrap();
        assert_eq!(result, http_url);
    }

    #[test]
    fn test_graph_api_base_constant() {
        assert_eq!(GRAPH_API_BASE, "https://graph.microsoft.com/v1.0");
    }
}
