use async_trait::async_trait;
use pdfium_render::prelude::*;
use std::path::Path;
use std::collections::HashMap;
use super::document_ingestor::{
    DocumentIngestor,
    IngestedDocument,
    DocumentMetadata,
    IngestError
};

pub struct PdfIngestor;

#[async_trait]
impl DocumentIngestor for PdfIngestor {
    fn can_handle(&self, path: &Path) -> bool {
        path.extension()
            .map(|ext| ext.eq_ignore_ascii_case("pdf"))
            .unwrap_or(false)
    }

    async fn ingest_file(&self, path: &Path) -> Result<IngestedDocument, IngestError> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./resources/"))
                .or_else(|_| Pdfium::bind_to_system_library())
                .unwrap() // Or use the ? unwrapping operator to pass any error up to the caller
        );

        let document = pdfium.load_pdf_from_file(path, None)
            .map_err(|e| IngestError::Parse(e.to_string()))?;

        let mut extracted_text = String::new();

        for (index, page) in document.pages().iter().enumerate() {
            if let Ok(text) = page.text() {
                extracted_text.push_str(&format!("\n=============== Page {} ===============\n", index + 1));
                extracted_text.push_str(&text.all());
            }
        }

        Ok(IngestedDocument {
            title: path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            content: extracted_text,
            metadata: DocumentMetadata {
                source_type: "pdf".to_string(),
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
