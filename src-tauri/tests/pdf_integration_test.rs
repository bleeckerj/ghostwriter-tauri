#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use std::path::PathBuf;
use tokio;

// Include necessary modules
#[path = "../src/embeddings.rs"]
mod embeddings;
#[path = "../src/document_store.rs"]
mod document_store;
#[path = "../src/ingest/mod.rs"]
mod ingest;

use document_store::DocumentStore;
use ingest::pdf_ingestor::PdfIngestor;
use embeddings::EmbeddingGenerator;

#[tokio::test]
async fn test_full_pdf_pipeline() {
    // Set up test directories
    let test_db_path = PathBuf::from("./test_db");
    let test_pdf_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test.pdf");

    // Initialize store
    let mut doc_store = DocumentStore::new(test_db_path.clone())
        .expect("Failed to create test document store");

    // Register the PDF ingestor
    doc_store.register_ingestor(Box::new(PdfIngestor));

    // Process document
    let result = doc_store.process_document(&test_pdf_path).await;
    assert!(result.is_ok(), "Document processing failed: {:?}", result.err());

    // Clean up test database
    std::fs::remove_dir_all(test_db_path).ok();
}