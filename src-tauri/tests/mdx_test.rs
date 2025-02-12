use std::path::PathBuf;
#[path = "../src/ingest/mod.rs"]
mod ingest;
use ingest::mdx_ingestor::MdxIngestor;

#[tokio::test]
async fn test_mdx_pipeline() {
    // MDX-specific tests
}