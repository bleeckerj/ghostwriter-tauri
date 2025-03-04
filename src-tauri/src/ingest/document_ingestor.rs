#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use std::collections::HashMap;
use gray_matter::Pod;
use url::Url;  // Add the url crate to your Cargo.toml
use std::fs;

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

#[derive(Debug, Clone)]
pub enum Resource {
    FilePath(PathBuf),
    Url(String),
}

impl Resource {
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Resource::FilePath(path) => Some(path),
            _ => None,
        }
    }
    
    pub fn as_url(&self) -> Option<&str> {
        match self {
            Resource::Url(url) => Some(url),
            _ => None,
        }
    }
    
    // Helper method to extract file content
    pub async fn read_content(&self) -> Result<String, IngestError> {
        match self {
            Resource::FilePath(path) => {
                fs::read_to_string(path).map_err(IngestError::Io)
            },
            Resource::Url(url) => {
                Err(IngestError::UnsupportedFormat(format!("Cannot read content directly from URL: {}", url)))
            }
        }
    }
}

#[async_trait]
pub trait DocumentIngestor: Send + Sync + std::fmt::Debug {
    /// Check if this ingestor can handle the given resource
    fn can_handle(&self, resource: &Resource) -> bool;
    
    /// Process a single resource and return its content
    async fn ingest(&self, resource: &Resource) -> Result<IngestedDocument, IngestError>;
}