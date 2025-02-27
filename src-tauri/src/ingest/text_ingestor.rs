use async_trait::async_trait;
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use gray_matter::Pod;
use super::document_ingestor::{
    DocumentIngestor,
    IngestedDocument,
    DocumentMetadata,
    IngestError
};

#[derive(Debug)]
pub struct TextIngestor;

#[async_trait]
impl DocumentIngestor for TextIngestor {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension()
            .map(|ext| ext.eq_ignore_ascii_case("txt"))
            .unwrap_or(false)
    }

    async fn ingest_file(&self, path: &Path) -> Result<IngestedDocument, IngestError> {
        let content = fs::read_to_string(path)
            .map_err(IngestError::Io)?;

        let title = path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());

        Ok(IngestedDocument {
            title,
            content,
            metadata: DocumentMetadata {
                source_type: "text".to_string(),
                source_path: path.to_string_lossy().to_string(),
                author: None,
                created_date: None,
                modified_date: None,
                frontmatter: HashMap::new(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_text_ingestion() {
        // Create a temporary test file
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("fixtures");
        tokio::fs::create_dir_all(&path).await.unwrap();
        path.push("test.txt");

        let mut file = File::create(&path).await.unwrap();
        file.write_all(b"This is a test text file.\nWith multiple lines.").await.unwrap();

        let ingestor = TextIngestor;
        let result = ingestor.ingest_file(&path).await;

        // Clean up the test file
        tokio::fs::remove_file(&path).await.unwrap();

        assert!(result.is_ok(), "Text ingestion failed: {:?}", result.err());

        let doc = result.unwrap();
        assert_eq!(doc.title, "test.txt");
        assert_eq!(doc.content, "This is a test text file.\nWith multiple lines.");
        assert_eq!(doc.metadata.source_type, "text");
        assert_eq!(doc.metadata.source_path, path.to_string_lossy());
        assert!(doc.metadata.author.is_none());
        assert!(doc.metadata.created_date.is_none());
        assert!(doc.metadata.modified_date.is_none());
        assert!(doc.metadata.frontmatter.is_empty());
    }
}