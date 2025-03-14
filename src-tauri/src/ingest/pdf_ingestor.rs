use async_trait::async_trait;
use pdfium_render::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::any::Any;
use super::document_ingestor::{
    DocumentIngestor,
    IngestedDocument,
    DocumentMetadata,
    IngestError,
    Resource  // Add this import
};

#[derive(Debug)]
pub struct PdfIngestor;



#[async_trait]
impl DocumentIngestor for PdfIngestor {
    // Update can_handle to work with Resource enum
    fn can_handle(&self, resource: &Resource) -> bool {
        match resource {
            Resource::FilePath(path) => path.extension()
            .map(|ext| ext.eq_ignore_ascii_case("pdf"))
            .unwrap_or(false),
            Resource::Url(_) => false, // PDFs aren't handled via URL directly
            Resource::Database(_) => false,
        }
    }
    
    // Add the ingest method required by the trait
    async fn ingest(&self, resource: &Resource) -> Result<IngestedDocument, IngestError> {
        match resource {
            Resource::FilePath(path) => self.ingest_file(path).await,
            Resource::Url(url) => Err(IngestError::UnsupportedFormat(
                format!("PdfIngestor cannot process URLs directly: {}", url)
            )),
            Resource::Database(_) => Err(IngestError::UnsupportedFormat(
                "PdfIngestor cannot process database resources".to_string()
            )),
        }
    }
    fn as_any(&self) -> &dyn Any {
        self // This returns a reference to self as a type-erased &dyn Any
    }
}

// Move existing implementation to a helper method in a separate impl block
impl PdfIngestor {
    // Keep the existing ingest_file implementation
    async fn ingest_file(&self, path: &Path) -> Result<IngestedDocument, IngestError> {
        // Try loading the PDF library, with proper error handling
        crate::get_resource_dir_path();

        let pdfium_dir = if let Some(resource_dir_path) = crate::get_resource_dir_path() {
            log::debug!("Using globally stored resource directory for libpdfium: {:?}", resource_dir_path);
            resource_dir_path.join("resources")
        } else {
            log::warn!("Resource directory not found, attempting to load PDFium from system library");
            PathBuf::from("./Resources/resources")
        };

        let pdfium = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&pdfium_dir))
        .or_else(|err| {
            // Log the first failure
            log::warn!("Failed to load PDFium from resources/: {}", err);
            log::info!("Attempting to load PDFium from system library...");
            
            // Try the system library
            Pdfium::bind_to_system_library()
            .map_err(|sys_err| {
                // Log the second failure too
                log::error!("Failed to load PDFium from system library: {}", sys_err);
                log::error!("Could not initialize PDF processing engine");
                
                // Return the second error
                sys_err
            })
        })
        .map(Pdfium::new)
        .map_err(|e| log::error!("Failed to initialize PDF library: {}", e));
        
        // log::debug!("PDFium library successfully loaded");
        
        // Now try to load the PDF file
        let pdfium = match pdfium {
            Ok(pdfium) => pdfium,
            Err(_) => return Err(IngestError::Parse("Failed to initialize PDFium".to_string())),
        };

        let document = match pdfium.load_pdf_from_file(path, None) {
            Ok(doc) => doc,
            Err(e) => {
                log::error!("Failed to load PDF file {}: {}", path.display(), e);
                return Err(IngestError::Parse(format!("Failed to load PDF file: {}", e)));
            }
        };
        
        log::info!("Successfully loaded PDF file: {}", path.display());
        
        let mut extracted_text = String::new();
        
        for (index, page) in document.pages().iter().enumerate() {
            if let Ok(text) = page.text() {
                //extracted_text.push_str(&format!("\n=============== Page {} ===============\n", index + 1));
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
    
    // #[tokio::test]
    // async fn test_pdf_ingestion() {
    //     let ingestor = PdfIngestor;
    //     let pdf_path = get_test_pdf_path();
    
    //     let result = ingestor.ingest_file(&pdf_path).await;
    //     assert!(result.is_ok(), "PDF ingestion failed: {:?}", result.err());
    
    //     let doc = result.unwrap();
    //     println!("Content length: {}", doc.content.len());
    //     println!("First 100 chars: {}", &doc.content[..100.min(doc.content.len())]);
    // }
}
