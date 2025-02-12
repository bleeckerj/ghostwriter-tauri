#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
#[path = "../src/ingest/mod.rs"]
mod ingest;
use ingest::{DocumentIngestor, MdxIngestor, PdfIngestor, MarkdownIngestor};
use std::path::{Path, PathBuf};

#[tokio::test]
async fn test_ingestors_handle_correct_files() {
    let mdx = MdxIngestor;
    let pdf = PdfIngestor;
    let md = MarkdownIngestor;

    let test_files = vec![
        ("test.mdx", &mdx as &dyn DocumentIngestor),
        ("test.pdf", &pdf as &dyn DocumentIngestor),
        ("test.md", &md as &dyn DocumentIngestor),
    ];

    for (filename, ingestor) in test_files {
        assert!(ingestor.can_handle(Path::new(filename)));
    }
}