#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use async_trait::async_trait;
use gray_matter::{Matter, engine::YAML, Pod};

use super::document_ingestor::{DocumentIngestor, IngestedDocument, DocumentMetadata, IngestError, Resource};

#[derive(Debug)]
pub struct MdxIngestor;

/// Macro to safely extract nested values from `Pod` within a `HashMap<String, Pod>`
macro_rules! pod_get {
    ($map:expr, $key:expr, Array, $idx:expr, Hash, $subkey:expr, String) => {
        match $map.get($key) {
            Some(Pod::Array(arr)) => match arr.get($idx) {
                Some(Pod::Hash(sub_map)) => match sub_map.get($subkey) {
                    Some(Pod::String(value)) => Some(value.clone()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    };
}

#[async_trait]
impl DocumentIngestor for MdxIngestor {
    // Updated can_handle to work with Resource enum
    fn can_handle(&self, resource: &Resource) -> bool {
        match resource {
            Resource::FilePath(path) => path.extension()
                .map(|ext| ext.eq_ignore_ascii_case("mdx"))
                .unwrap_or(false),
            Resource::Url(_) => false, // This ingestor doesn't handle URLs
            Resource::Database(_) => false, // This ingestor doesn't handle databases
        }
    }

    // New method to implement the trait
    async fn ingest(&self, resource: &Resource) -> Result<IngestedDocument, IngestError> {
        match resource {
            Resource::FilePath(path) => self.ingest_file(path).await,
            Resource::Url(url) => Err(IngestError::UnsupportedFormat(
                format!("MdxIngestor cannot process URLs: {}", url)
            )),
            Resource::Database(_) => Err(IngestError::UnsupportedFormat(
                "MdxIngestor cannot process database resources".to_string()
            )),
        }
    }
}

// Helper methods in a separate impl block
impl MdxIngestor {
    // Keep the existing ingest_file implementation
    async fn ingest_file(&self, path: &Path) -> Result<IngestedDocument, IngestError> {
        let content = fs::read_to_string(path)
            .map_err(IngestError::Io)?;

        let matter = Matter::<YAML>::new();
        let result = matter.parse(&content);
        
        let (content, frontmatter) = match result.data {
            Some(data) => (result.content, data),
            None => (content, Pod::Hash(HashMap::new())),
        };

        let metadata = DocumentMetadata {
            source_type: "mdx".to_string(),
            source_path: path.to_string_lossy().to_string(),
            author: None, 
            created_date: None,
            modified_date: None,
            frontmatter: match frontmatter {
                Pod::Hash(map) => map,
                _ => HashMap::new(),
            },
        };

        if let Some(Pod::Hash(content_metadata)) = metadata.frontmatter.get("contentMetadata") {
            println!("Found contentMetadata: {:?}", content_metadata);
            
            // Example: Extracting the "title" field from contentMetadata
            if let Some(Pod::String(title)) = content_metadata.get("title") {
                println!("Title: {}", title);
            }
        }

        Ok(IngestedDocument {
            title: path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            content,
            metadata,
        })
    }
}