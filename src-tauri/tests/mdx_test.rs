use std::path::{Path, PathBuf};
use gray_matter::Pod;
// Only import what we need for MDX testing
#[path = "../src/ingest/document_ingestor.rs"]
mod document_ingestor;
#[path = "../src/ingest/mdx_ingestor.rs"]
mod mdx_ingestor;

use document_ingestor::DocumentIngestor;
use mdx_ingestor::MdxIngestor;

#[tokio::test]
async fn test_mdx_pipeline() {
    // Set up test MDX path
    let test_mdx_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test.mdx");

    // Initialize ingestor
    let ingestor = MdxIngestor;

    // Test file type recognition
    assert!(ingestor.can_handle(&test_mdx_path));
    assert!(!ingestor.can_handle(Path::new("test.txt")));
    assert!(!ingestor.can_handle(Path::new("test.pdf")));

    // Test MDX ingestion
    let result = ingestor.ingest_file(&test_mdx_path).await;
    assert!(result.is_ok(), "MDX ingestion failed: {:?}", result.err());

    let document = result.unwrap();
    
    // Verify document structure
    assert_eq!(document.metadata.source_type, "mdx");
    assert!(!document.content.is_empty(), "MDX content should not be empty");
    
    // Verify frontmatter parsing
    assert_eq!(document.title, "test.mdx");
    assert_eq!(document.metadata.frontmatter.get("author").unwrap(), "Julian Bleecker");

    // Get the authors array as a string and parse it
    let authors = document.metadata.frontmatter.get("authors").expect("Should have authors");

    println!("Authors: {:?}", authors);    
    println!("authors[0] {:?}", authors[0]);
    println!("authors[0] firstName {:?}", authors[0]["firstName"]);

    // Print first 100 chars of content for visual inspection
    println!("Content preview: {}", &document.content[..100.min(document.content.len())]);
}