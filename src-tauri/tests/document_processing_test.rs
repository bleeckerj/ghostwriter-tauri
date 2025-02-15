#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use std::path::{Path, PathBuf};
use dotenv::dotenv;
use std::env;
// Include necessary modules
#[path = "../src/embeddings.rs"]
mod embeddings;
#[path = "../src/document_store.rs"]
mod document_store;
#[path = "../src/ingest/mod.rs"]
mod ingest;
use document_store::DocumentStore;
use ingest::pdf_ingestor::PdfIngestor;
use ingest::mdx_ingestor::MdxIngestor;

use embeddings::EmbeddingGenerator;

async fn setup_store() -> (DocumentStore, EmbeddingGenerator) {
    // Load environment variables from project root
    dotenv().ok();
    
    let openai_key = env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY must be set in .env file");
    
    //let temp_dir = tempfile::tempdir().unwrap();

    let temp_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("fixtures");

    println!("Temp dir: {:?}", temp_dir);
    let embedding_generator: EmbeddingGenerator = EmbeddingGenerator::from_api_key(&openai_key);
    let cloned_embedding_generator = embedding_generator.clone();
    let mut store = DocumentStore::new(temp_dir.to_path_buf(), embedding_generator.into()).unwrap();
    
    // Register ingestors
    store.register_ingestor(Box::new(MdxIngestor));
    store.register_ingestor(Box::new(PdfIngestor));
    
    
    (store, cloned_embedding_generator)
}

#[tokio::test]
async fn test_mdx_processing() {
    let (mut store, embedding_generator) = setup_store().await;
    
    // Use your test MDX file
    let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("fixtures")
    .join("test.mdx");
    
    // Process the document
    let result = store.process_document(&test_path, &embedding_generator).await;
    assert!(result.is_ok(), "Document processing failed: {:?}", result.err());
    
    // Verify document was stored
    let docs = store.fetch_documents().unwrap();
    assert_eq!(docs.documents.len(), 1);
    
    // Verify document metadata
    let doc = &docs.documents[0];
    assert!(doc.id > 0);
    assert_eq!(doc.name, "test.mdx");
    
    // Get embeddings count from database
    let conn = store.get_connection();
    let embedding_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM embeddings WHERE doc_id = ?1",
        [doc.id],
        |row| row.get(0)
    ).unwrap();
    
    assert!(embedding_count > 0, "No embeddings were generated");
}

#[tokio::test]
async fn test_pdf_processing() {
    let store = setup_store().await;
    let mut document_store = store.0;
    let embedding_generator = store.1;
    
    // Use your test PDF file
    let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("fixtures")
    .join("test.pdf");
    
    // Process the document
    let result = document_store.process_document(&test_path, &embedding_generator).await;
    assert!(result.is_ok(), "Document processing failed: {:?}", result.err());
    
    // Verify document was stored
    let docs = document_store.fetch_documents().unwrap();
    assert_eq!(docs.documents.len(), 1);
    
    // Get embeddings count
    let conn = document_store.get_connection();
    let embedding_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM embeddings WHERE doc_id = ?1",
        [docs.documents[0].id],
        |row| row.get(0)
    ).unwrap();
    
    assert!(embedding_count > 0, "No embeddings were generated");
}