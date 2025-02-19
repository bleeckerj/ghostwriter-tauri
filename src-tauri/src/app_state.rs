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
use crate::preferences::Preferences;


pub struct AppState {
    pub doc_store: Arc<DocumentStore>,
    pub embedding_generator: Arc<EmbeddingGenerator>,
    pub conversation: Mutex<Conversation>,
    pub buffer: Mutex<String>,
    pub logger: Arc<Mutex<Logger>>,  
    pub api_key: Mutex<Option<String>>, 
    pub preferences: Mutex<Preferences>,  // Store preferences in memory
}

impl AppState {
    pub fn new(
        doc_store: DocumentStore,
        embedding_generator: EmbeddingGenerator,
        initial_log_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let logger = Logger::new(initial_log_path)?;
        
        // Create AppState first without preferences
        let app_state = Self {
            logger: Arc::new(Mutex::new(logger)),
            doc_store: Arc::new(doc_store),
            embedding_generator: Arc::new(embedding_generator),
            conversation: Mutex::new(Conversation::new(16000)),
            buffer: Mutex::new(String::new()),
            api_key: Mutex::new(None),
            preferences: Mutex::new(Preferences::default()), // Start with default preferences
        };
        Ok(app_state)
    }


    // ✅ Load API key from a file
    pub async fn load_api_key(&self, app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        let path = app_handle
            .path()
            .app_local_data_dir()
            .unwrap_or_default()
            .join("api_key.txt");

        if let Ok(contents) = fs::read_to_string(&path) {
            let mut api_key = self.api_key.lock().await;
            *api_key = Some(contents.trim().to_string());
            println!("Loaded API Key: {}", contents.trim());
        }

        Ok(())
    }

    // ✅ Save API key to a file
    pub async fn save_api_key(&self, _app_handle: &AppHandle, key: String) -> Result<(), Box<dyn std::error::Error>> {
        let env_path = ".env"; // ✅ Save to .env in the app's root directory

        // ✅ Read existing .env contents (if any)
        let mut env_contents = fs::read_to_string(env_path).unwrap_or_else(|_| String::new());

        // ✅ Remove any existing `OPENAI_API_KEY` entry
        env_contents = env_contents
            .lines()
            .filter(|line| !line.starts_with("OPENAI_API_KEY="))
            .map(|line| format!("{}\n", line))
            .collect();

        // ✅ Append the new API key entry
        env_contents.push_str(&format!("OPENAI_API_KEY={}\n", key));

        // ✅ Write back to .env
        fs::write(env_path, env_contents)?;

        let mut api_key = self.api_key.lock().await; // ✅ Store it in-memory as well
        *api_key = Some(key);

        println!("API Key saved to .env.");
        Ok(())
    }
}
