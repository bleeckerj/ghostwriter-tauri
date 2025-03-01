#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use super::*;
use std::path::Path;
use std::fs;
use std::collections::HashMap;

use async_trait::async_trait;
use tokio::fs::metadata;

#[derive(Debug)]
pub struct MarkdownIngestor;

#[async_trait]
impl DocumentIngestor for MarkdownIngestor {
    fn can_handle(&self, resource: &Resource) -> bool {
        match resource {
            Resource::FilePath(path) => path.extension().map_or(false, |ext| ext == "md"),
            Resource::Url(_) => false, // This ingestor doesn't handle URLs
        }
    }

    async fn ingest(&self, resource: &Resource) -> Result<IngestedDocument, IngestError> {
        match resource {
            Resource::FilePath(path) => {
                let content = fs::read_to_string(path).map_err(IngestError::Io)?;
                
                Ok(IngestedDocument {
                    title: path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    content,
                    metadata: DocumentMetadata {
                        source_type: "markdown".to_string(),
                        source_path: path.to_string_lossy().to_string(),
                        author: None,
                        created_date: None,
                        modified_date: None,
                        frontmatter: HashMap::new(),
                    }
                })
            },
            Resource::Url(url) => Err(IngestError::UnsupportedFormat(format!(
                "URL ingestion not supported for Markdown: {}", url
            ))),
        }
    }
}