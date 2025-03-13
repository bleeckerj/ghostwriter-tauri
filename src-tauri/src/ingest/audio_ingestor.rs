use async_trait::async_trait;
use simple_transcribe_rs::model_handler;
use simple_transcribe_rs::transcriber;
use std::path::Path;
use std::collections::HashMap;
use super::document_ingestor::{
    DocumentIngestor,
    IngestedDocument,
    DocumentMetadata,
    IngestError,
    Resource
};
use chrono::Utc;

#[derive(Debug)]
pub struct AudioIngestor;

#[async_trait]
impl DocumentIngestor for AudioIngestor {
    fn can_handle(&self, resource: &Resource) -> bool {
        match resource {
            Resource::FilePath(path) => {
                let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                matches!(extension, "mp3" | "wav" | "flac" | "aac" | "ogg")
            },
            Resource::Url(_) => false, // This ingestor doesn't handle URLs
            Resource::Database(_) => false, // This ingestor doesn't handle databases
        }
    }

    async fn ingest(&self, resource: &Resource) -> Result<IngestedDocument, IngestError> {
        match resource {
            Resource::FilePath(path) => self.ingest_file(path).await,
            Resource::Url(url) => Err(IngestError::UnsupportedFormat(
                format!("WhisperIngestor cannot process URLs: {}", url)
            )),
            Resource::Database(_) => Err(IngestError::UnsupportedFormat(
                "WhisperIngestor cannot process database resources".to_string()
            )),
        }
    }
}

impl AudioIngestor {
    async fn ingest_file(&self, path: &Path) -> Result<IngestedDocument, IngestError> {
        // Initialize the Whisper library
        let m = model_handler::ModelHandler::new("tiny", "models/").await;
        let trans = transcriber::Transcriber::new(m);

        let result = trans.transcribe(path.to_string_lossy().as_ref(), None).unwrap();
        let transcription = result.get_text();
        
        Ok(IngestedDocument {
            title: path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            content: transcription.to_string(),
            metadata: DocumentMetadata {
                source_type: "audio".to_string(),
                source_path: path.to_string_lossy().to_string(),
                author: None,
                created_date: Some(Utc::now().naive_utc().to_string()),  // Set created_date to the current date
                modified_date: Some(Utc::now().naive_utc().to_string()),  // Set created_date to the current date
                frontmatter: HashMap::new(),
            }
        })
    }
}