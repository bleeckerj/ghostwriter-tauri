#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use async_trait::async_trait;
use pdf_extract::extract_text;
use super::{DocumentIngestor, IngestedDocument, DocumentMetadata, IngestError};
use std::path::Path;

pub struct PdfIngestor;

#[async_trait]
impl DocumentIngestor for PdfIngestor {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension()
            .map(|ext| ext.eq_ignore_ascii_case("pdf"))
            .unwrap_or(false)
    }

    async fn ingest_file(&self, path: &Path) -> Result<IngestedDocument, IngestError> {
        let content = pdf_extract::extract_text(path)
            .map_err(|e| IngestError::Parse(e.to_string()))?;

        Ok(IngestedDocument {
            title: path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            content,
            metadata: DocumentMetadata {
                source_type: "pdf".to_string(),
                source_path: path.to_string_lossy().to_string(),
                author: None,
                created_date: None,
                modified_date: None,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_test_pdf_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("test.pdf")
    }

    #[tokio::test]
    async fn test_pdf_ingestion() {
        let ingestor = PdfIngestor;
        let pdf_path = get_test_pdf_path();
        
        let result = ingestor.ingest_file(&pdf_path).await;
        assert!(result.is_ok(), "PDF ingestion failed: {:?}", result.err());
        
        let doc = result.unwrap();
        println!("Content length: {}", doc.content.len());
        println!("First 100 chars: {}", &doc.content[..100.min(doc.content.len())]);
    }
}