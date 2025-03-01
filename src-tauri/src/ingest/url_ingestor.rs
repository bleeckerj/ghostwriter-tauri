use async_trait::async_trait;
use reqwest::Client;
use std::path::Path;
use std::collections::HashMap;
use gray_matter::Pod;
use regex::Regex;
use super::*;
use log;

#[derive(Debug, Clone)]
pub struct UrlDocumentIngestor;

impl UrlDocumentIngestor {
    pub async fn ingest_url(&self, url: &str) -> Result<IngestedDocument, IngestError> {
        // 1. Construct the Jina Reader URL
        let encoded_url = reqwest::Url::parse(url)
            .map_err(|e| IngestError::Parse(format!("Invalid URL: {}", e)))?;
        let jina_api_url = format!("https://r.jina.ai/{}", encoded_url);
        
        log::info!("Fetching content from URL: {} via Jina Reader", url);
        
        // 2. Create a client with a timeout
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            // .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .build()
            .map_err(|e| IngestError::Parse(format!("Failed to build HTTP client: {}", e)))?;
        
        // 3. Send GET request
        log::debug!("Sending request to: {}", jina_api_url);
        let response = client.get(&jina_api_url)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to connect to Jina Reader API: {}", e);
                IngestError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to connect to Jina Reader API: {}", e)
                ))
            })?;

        // 4. Check status before getting content
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            let error_msg = format!("Jina Reader API returned error {}: {}", status, error_text);
            log::error!("{}", error_msg);
            return Err(IngestError::Parse(error_msg));
        }

        // 5. Get the Markdown/text response
        let response_text = response.text().await
            .map_err(|e| IngestError::Parse(format!("Failed to read response body: {}", e)))?;
        
        // 6. Clean any HTML tags from the Markdown
        let re = Regex::new(r"<[^>]*>").unwrap(); // Regex to match HTML tags
        let cleaned_content = re.replace_all(&response_text, "").to_string();
        
        // 7. Extract title from first line if possible
        let mut title = url.to_string();
        let first_line = cleaned_content.lines().next();
        if let Some(line) = first_line {
            let trimmed = line.trim();
            // If line starts with # or ## it's likely a title
            if (trimmed.starts_with("# ") || trimmed.starts_with("## ")) && trimmed.len() > 3 {
                title = trimmed[2..].trim().to_string();
            }
        }
        
        // 8. Create document
        let mut frontmatter = HashMap::new();
        frontmatter.insert("url".to_string(), Pod::String(url.to_string()));
        
        Ok(IngestedDocument {
            title,  // Use extracted title or URL
            content: cleaned_content,
            metadata: DocumentMetadata {
                source_type: "URL".to_string(),
                source_path: url.to_string(),
                author: None,
                created_date: None,
                modified_date: None,
                frontmatter,
            },
        })
    }
}

#[async_trait]
impl DocumentIngestor for UrlDocumentIngestor {
    fn can_handle(&self, resource: &Resource) -> bool {
        match resource {
            Resource::FilePath(_) => false,
            Resource::Url(url) => url.starts_with("http"),
        }
    }

    async fn ingest(&self, resource: &Resource) -> Result<IngestedDocument, IngestError> {
        match resource {
            Resource::Url(url) => self.ingest_url(url).await,
            Resource::FilePath(path) => Err(IngestError::UnsupportedFormat(
                format!("UrlDocumentIngestor cannot process file paths: {}", path.display())
            )),
        }
    }
}