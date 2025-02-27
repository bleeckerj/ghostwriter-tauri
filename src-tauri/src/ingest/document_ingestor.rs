#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use std::collections::HashMap;
use gray_matter::Pod;

#[derive(Error, Debug)]
pub enum IngestError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

#[derive(Debug)]
pub struct IngestedDocument {
    pub title: String,
    pub content: String,
    pub metadata: DocumentMetadata,
}

#[derive(Debug)]
pub struct DocumentMetadata {
    pub source_type: String,
    pub source_path: String,
    pub author: Option<String>,
    pub created_date: Option<String>,
    pub modified_date: Option<String>,
    // Change to Pod to handle complex YAML structures
    pub frontmatter: HashMap<String, Pod>,
}

#[async_trait]
pub trait DocumentIngestor: Send + Sync + std::fmt::Debug {
    /// Check if this ingestor can handle the given file
    fn can_handle(&self, path: &Path) -> bool;
    
    /// Process a single file and return its content
    async fn ingest_file(&self, path: &Path) -> Result<IngestedDocument, IngestError>;
}