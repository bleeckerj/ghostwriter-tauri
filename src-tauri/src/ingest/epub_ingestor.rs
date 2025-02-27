use async_trait::async_trait;
use epub::doc::EpubDoc;
use regex::Regex;
use std::path::Path;
use std::collections::HashMap;
use super::document_ingestor::{
    DocumentIngestor,
    IngestedDocument,
    DocumentMetadata,
    IngestError
};
use gray_matter::Pod;
#[derive(Debug)]
pub struct EpubIngestor;

#[async_trait]
impl DocumentIngestor for EpubIngestor {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension()
            .map(|ext| ext.eq_ignore_ascii_case("epub"))
            .unwrap_or(false)
    }

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
                //content.push_str(&format!("\n=============== Chapter {} ===============\n", i + 1));
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
                modified_date: None, // EPUB doesn't typically have a modified date
                frontmatter,
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

    fn get_test_epub_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("test.epub") // Replace with your test epub file
    }

    #[tokio::test]
    async fn test_epub_ingestion() {
        // Create a temporary test file
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("fixtures");
        tokio::fs::create_dir_all(&path).await.unwrap();
        path.push("test.epub");

        // Create a dummy epub file (replace with a real epub if needed)
        let mut file = File::create(&path).await.unwrap();
        file.write_all(b"Dummy EPUB Content").await.unwrap();

        let ingestor = EpubIngestor;
        let result = ingestor.ingest_file(&path).await;

        // Clean up the test file
        tokio::fs::remove_file(&path).await.unwrap();

        assert!(result.is_ok(), "EPUB ingestion failed: {:?}", result.err());

        let doc = result.unwrap();
        println!("Content length: {}", doc.content.len());
        println!("First 100 chars: {}", &doc.content[..100.min(doc.content.len())]);
    }
}