use std::path::Path;
use walkdir::WalkDir;
use futures::stream::{self, StreamExt};

pub struct DocumentScanner {
    ingestors: Vec<Box<dyn DocumentIngestor>>,
}

impl DocumentScanner {
    pub fn new() -> Self {
        Self {
            ingestors: Vec::new(),
        }
    }

    pub fn register_ingestor(&mut self, ingestor: Box<dyn DocumentIngestor>) {
        self.ingestors.push(ingestor);
    }

    pub async fn scan_directory(&self, dir: &Path) -> Result<Vec<IngestedDocument>, IngestError> {
        let mut documents = Vec::new();

        for entry in WalkDir::new(dir).follow_links(true).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            
            // Find appropriate ingestor
            if let Some(ingestor) = self.ingestors.iter().find(|i| i.can_handle(path)) {
                match ingestor.ingest_file(path).await {
                    Ok(doc) => documents.push(doc),
                    Err(e) => eprintln!("Failed to ingest {}: {}", path.display(), e),
                }
            }
        }

        Ok(documents)
    }
}