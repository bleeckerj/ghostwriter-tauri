use std::path::Path;
use std::fs;
use std::collections::HashMap;
use async_trait::async_trait;
use gray_matter::{Matter, engine::YAML, Pod};

use super::*;  // This gives us access to DocumentIngestor trait and other types

pub struct MdxIngestor;

#[async_trait]
impl DocumentIngestor for MdxIngestor {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension()
            .map(|ext| ext.eq_ignore_ascii_case("mdx"))
            .unwrap_or(false)
    }

    async fn ingest_file(&self, path: &Path) -> Result<IngestedDocument, IngestError> {
        let content = fs::read_to_string(path)
            .map_err(IngestError::Io)?;

        let matter = Matter::<YAML>::new();
        let result = matter.parse(&content);
        
        let (content, metadata, frontmatter) = if let Some(data) = result.data {
            let mut frontmatter = HashMap::new();
            
            if let Pod::Hash(map) = data {
                // Convert all frontmatter fields to strings
                for (key, value) in map {
                    if let Pod::String(val) = value {
                        frontmatter.insert(key, val);
                    }
                }
            }

            (
                result.content,
                DocumentMetadata {
                    source_type: "mdx".to_string(),
                    source_path: path.to_string_lossy().to_string(),
                    author: frontmatter.get("author").cloned(),
                    created_date: frontmatter.get("date").cloned(),
                    modified_date: None,
                },
                Some(frontmatter)
            )
        } else {
            (
                content,
                DocumentMetadata {
                    source_type: "mdx".to_string(),
                    source_path: path.to_string_lossy().to_string(),
                    author: None,
                    created_date: None,
                    modified_date: None,
                },
                None
            )
        };

        Ok(IngestedDocument {
            title: frontmatter
                .and_then(|f| f.get("title").cloned())
                .unwrap_or_else(|| path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()),
            content,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_can_handle_mdx_files() {
        let ingestor = MdxIngestor;
        assert!(ingestor.can_handle(Path::new("test.mdx")));
        assert!(ingestor.can_handle(Path::new("test.MDX")));
        assert!(!ingestor.can_handle(Path::new("test.md")));
        assert!(!ingestor.can_handle(Path::new("test.txt")));
    }

    #[tokio::test]
    async fn test_mdx_with_frontmatter() {
        let ingestor = MdxIngestor;
        let mut temp_file = NamedTempFile::new().unwrap();
        
        write!(temp_file, r#"---
title: Test Document
author: John Doe
date: 2024-02-11
tags: ["test", "mdx"]
---
# Main Content
This is the test content."#).unwrap();

        let result = ingestor.ingest_file(temp_file.path()).await.unwrap();
        
        assert_eq!(result.title, "Test Document");
        assert_eq!(result.metadata.author.unwrap(), "John Doe");
        assert_eq!(result.metadata.created_date.unwrap(), "2024-02-11");
        assert!(result.content.contains("# Main Content"));
    }

    #[tokio::test]
    async fn test_mdx_without_frontmatter() {
        let ingestor = MdxIngestor;
        let mut temp_file = NamedTempFile::new().unwrap();
        
        write!(temp_file, "# No Frontmatter\nJust content.").unwrap();

        let result = ingestor.ingest_file(temp_file.path()).await.unwrap();
        
        assert_eq!(result.metadata.author, None);
        assert_eq!(result.metadata.created_date, None);
        assert!(result.content.contains("# No Frontmatter"));
    }

    #[tokio::test]
    async fn test_mdx_with_partial_frontmatter() {
        let ingestor = MdxIngestor;
        let mut temp_file = NamedTempFile::new().unwrap();
        
        write!(temp_file, r#"---
title: Just Title
---
# Partial Frontmatter"#).unwrap();

        let result = ingestor.ingest_file(temp_file.path()).await.unwrap();
        
        assert_eq!(result.title, "Just Title");
        assert_eq!(result.metadata.author, None);
        assert_eq!(result.metadata.created_date, None);
        assert!(result.content.contains("# Partial Frontmatter"));
    }
}