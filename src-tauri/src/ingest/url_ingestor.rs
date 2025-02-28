use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::path::Path;
use std::collections::HashMap;
use gray_matter::Pod;
use super::document_ingestor::{DocumentIngestor, IngestedDocument, IngestError, DocumentMetadata};

#[derive(Deserialize)]
struct IngestResponse {
    content: String,
}

#[derive(Debug, Clone)]
pub struct UrlDocumentIngestor;

impl UrlDocumentIngestor {
    

    pub async fn ingest_url(&self, url: &str) -> Result<IngestedDocument, IngestError> {
        let client = Client::new();
        let response = client.post(url)
            .json(&serde_json::json!({ "url": url }))
            .send()
            .await
            .map_err(|e| IngestError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        if !response.status().is_success() {
            return Err(IngestError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Failed to fetch document")));
        }

        let ingest_data: IngestResponse = response.json()
            .await
            .map_err(|e| IngestError::Parse(e.to_string()))?;

        Ok(IngestedDocument {
            title: "Fetched Content".to_string(),
            content: ingest_data.content,
            metadata: DocumentMetadata {
                source_type: "URL".to_string(),
                source_path: url.to_string(),
                author: None,
                created_date: None,
                modified_date: None,
                frontmatter: HashMap::new(),
            },
        })
    }
}

#[async_trait]
impl DocumentIngestor for UrlDocumentIngestor {
    fn can_handle(&self, path: &Path) -> bool {
        false // This ingestor doesn't handle local files
    }

    async fn ingest_file(&self, path: &Path) -> Result<IngestedDocument, IngestError> {
        Err(IngestError::UnsupportedFormat("URLs are handled separately".to_string()))
    }
}