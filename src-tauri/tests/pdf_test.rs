use std::path::{Path, PathBuf};
// Only import what we need
#[path = "../src/ingest/document_ingestor.rs"]
mod document_ingestor;
#[path = "../src/ingest/pdf_ingestor.rs"]
mod pdf_ingestor;

use document_ingestor::DocumentIngestor;
use pdf_ingestor::PdfIngestor;

#[tokio::test]
async fn test_pdf_pipeline() {
    // Set up test PDF path
    let test_pdf_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test.pdf");

    // Initialize ingestor
    let ingestor = PdfIngestor;

    // Test file type recognition
    assert!(ingestor.can_handle(&test_pdf_path));
    assert!(!ingestor.can_handle(Path::new("test.txt")));
    assert!(!ingestor.can_handle(Path::new("test.md")));

    // Test PDF ingestion
    let result = ingestor.ingest_file(&test_pdf_path).await;
    assert!(result.is_ok(), "PDF ingestion failed: {:?}", result.err());

    let document = result.unwrap();
    
    // Verify document structure
    assert_eq!(document.metadata.source_type, "pdf");
    assert!(!document.content.is_empty(), "PDF content should not be empty");
    assert_eq!(
        document.metadata.source_path,
        test_pdf_path.to_string_lossy().to_string()
    );

    // Print first 100 chars of content for visual inspection
    println!("Content preview: {}", &document.content[..100.min(document.content.len())]);
}

#[tokio::test]
async fn test_pdf_invalid_file() {
    let ingestor = PdfIngestor;
    let invalid_path = PathBuf::from("nonexistent.pdf");
    
    let result = ingestor.ingest_file(&invalid_path).await;
    assert!(result.is_err(), "Should fail with invalid file");
}