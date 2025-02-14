#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use chrono::format;
use tauri::AppHandle;
use crate::conversations::Conversation;
//use crate::document_store::{self, DocumentStore};
use crate::document_store::DocumentStore;
use crate::embeddings::EmbeddingGenerator;
use crate::logger::Logger;
use std::sync::{Arc};
use tokio::sync::Mutex;
use std::path::PathBuf;
use tauri::Manager;
use std::fs;

pub struct AppState {
    pub doc_store: Arc<DocumentStore>,
    pub embedding_generator: Arc<EmbeddingGenerator>,
    pub conversation: Mutex<Conversation>,
    pub buffer: Mutex<String>,
    pub logger: Arc<Mutex<Logger>>,  // ✅ Now using Arc<Mutex<Logger>>
}

impl AppState {
    pub fn new(
        doc_store: DocumentStore,
        embedding_generator: EmbeddingGenerator,
        initial_log_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize logger
        let logger = Logger::new(initial_log_path)?;

        Ok(Self {
            logger: Arc::new(Mutex::new(logger)),  // ✅ Correct type
            doc_store: Arc::new(doc_store),
            embedding_generator: Arc::new(embedding_generator),
            conversation: Mutex::new(Conversation::new(16000)),
            buffer: Mutex::new(String::new()),
        })
        
    }


    // Add method to update logger path
    pub async fn update_logger_path(&self, app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        let log_path = app_handle
            .path()
            .app_local_data_dir()
            .unwrap_or(std::path::PathBuf::new())
            .join("log.json")  // This properly handles path separators
            .to_string_lossy()
            .to_string();  // Convert to owned String

        fs::create_dir(app_handle
            .path()
            .app_local_data_dir()
            .unwrap_or(std::path::PathBuf::new()))?;

        print!("Trying to update logger path to: {}", log_path);
        let new_logger = Logger::new(&log_path)?;
        
        let mut logger = self.logger.lock().await;  // ✅ Use `.await` to acquire the lock asynchronously
        *logger = new_logger;
        
        
        Ok(())
    }
}
