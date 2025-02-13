#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use super::*;
use std::path::Path;
use std::fs;
use std::collections::HashMap;

use async_trait::async_trait;
use tokio::fs::metadata;

pub struct MarkdownIngestor;

#[async_trait]
impl DocumentIngestor for MarkdownIngestor {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension()
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false)
    }

    async fn ingest_file(&self, path: &Path) -> Result<IngestedDocument, IngestError> {
        let content = fs::read_to_string(path)
            .map_err(IngestError::Io)?;
        println!("Content: {}", content);
        
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
    }
}