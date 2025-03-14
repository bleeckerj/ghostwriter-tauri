use async_trait::async_trait;
use epub::doc::EpubDoc;
use regex::Regex;
use std::path::Path;
use std::collections::HashMap;
use super::document_ingestor::{
    DocumentIngestor,
    IngestedDocument,
    DocumentMetadata,
    IngestError,
    Resource
};
use std::any::Any;
use gray_matter::Pod;

#[derive(Debug)]
pub struct EpubIngestor;

#[async_trait]
impl DocumentIngestor for EpubIngestor {
    // Implement the required can_handle method with Resource parameter
    fn can_handle(&self, resource: &Resource) -> bool {
        match resource {
            Resource::FilePath(path) => path.extension()
                .map(|ext| ext.eq_ignore_ascii_case("epub"))
                .unwrap_or(false),
            Resource::Url(_) => false, // This ingestor doesn't handle URLs
            Resource::Database(_) => false, // This ingestor doesn't handle databases
        }
    }
    
    // Implement the required ingest method with Resource parameter
    async fn ingest(&self, resource: &Resource) -> Result<IngestedDocument, IngestError> {
        match resource {
            Resource::FilePath(path) => self.ingest_file(path).await,
            Resource::Url(url) => Err(IngestError::UnsupportedFormat(
                format!("EpubIngestor cannot process URLs: {}", url)
            )),
            Resource::Database(_) => Err(IngestError::UnsupportedFormat(
                "EpubIngestor cannot process database resources".to_string()
            )),
        }
    }
    fn as_any(&self) -> &dyn Any {
        self // This returns a reference to self as a type-erased &dyn Any
    }
}

// Extend EpubIngestor with additional methods
impl EpubIngestor {
    // Helper method for file-specific ingestion logic
    async fn ingest_file(&self, path: &Path) -> Result<IngestedDocument, IngestError> {
        let mut book = EpubDoc::new(path)
            .map_err(|e| IngestError::Parse(e.to_string()))?;

        let title = book.mdata("title").unwrap_or_else(|| {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });

        let mut content = String::new();
        let num_pages = book.get_num_pages();
        let re = Regex::new(r"<[^>]*>").unwrap(); // Regex to match HTML tags
        for i in 0..num_pages {
            book.set_current_page(i);
            if let Some((chapter_content, _)) = book.get_current_str() {
                // Strip HTML tags from chapter_content
                let stripped_content = re.replace_all(&chapter_content, "");
                content.push_str(&stripped_content);
            }
        }

        let mut frontmatter: HashMap<String, Pod> = HashMap::new();
        for (key, value) in book.metadata.clone() {
            frontmatter.insert(key, Pod::String(value.join(", ")));
        }

        Ok(IngestedDocument {
            title,
            content,
            metadata: DocumentMetadata {
                source_type: "epub".to_string(),
                source_path: path.to_string_lossy().to_string(),
                author: book.mdata("creator"),
                created_date: book.mdata("date"),
                modified_date: None,
                frontmatter,
            }
        })
    }
}