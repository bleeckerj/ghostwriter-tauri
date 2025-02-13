#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use async_trait::async_trait;
use gray_matter::{Matter, engine::YAML, Pod};

use super::document_ingestor::{DocumentIngestor, IngestedDocument, DocumentMetadata, IngestError};

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use std::path::PathBuf;

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
        
        // Use the actual test file
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("test.mdx");

        let result = ingestor.ingest_file(&test_path).await.unwrap();
        
        // Print the actual values to help debug
        println!("Frontmatter content: {:?}", result.metadata.frontmatter);

        if let Some(first_name) = pod_get!(result.metadata.frontmatter, "authors", Array, 0, Hash, "firstName", String) {
            println!("First author's firstName: {}", first_name);
        } else {
            println!("Could not retrieve first author's firstName");
        }

        // Check the nested contentMetadata.title
        if let Some(Pod::Hash(content_metadata)) = result.metadata.frontmatter.get("contentMetadata") {
            if let Some(Pod::String(title)) = content_metadata.get("title") {
                println!("Found title: {}", title);
                assert_eq!(title, "the race for cyberspace information technology in the black diaspora");
            }
        }

        // Assert the values we expect from the actual file
        assert!(!result.content.is_empty(), "Content should not be empty");
        
        // Access and verify the nested structure
        if let Some(Pod::Array(authors)) = result.metadata.frontmatter.get("authors") {
            if let Some(Pod::Hash(first_author)) = authors.first() {
                if let Some(Pod::String(first_name)) = first_author.get("firstName") {
                    assert_eq!(first_name, "Ron");
                }
            }
        }
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

//     #[tokio::test]
//     async fn test_mdx_with_partial_frontmatter() {
//         let ingestor = MdxIngestor;
//         let mut temp_file = NamedTempFile::new().unwrap();
        
//         write!(temp_file, r#"---
// title: Just Title
// ---
// # Partial Frontmatter"#).unwrap();

//         let result = ingestor.ingest_file(temp_file.path()).await.unwrap();
        
//         assert_eq!(result.title, "Just Title");
//         assert_eq!(result.metadata.author, None);
//         assert_eq!(result.metadata.created_date, None);
//         assert!(result.content.contains("# Partial Frontmatter"));
//     }
}