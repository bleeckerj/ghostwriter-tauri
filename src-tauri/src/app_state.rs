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
use tokio::task;
use thiserror::Error;
use keyring::Entry;
use secrecy::{SecretString};


#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Logger creation error: {0}")]
    LoggerCreationError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Join error: {0}")]
    JoinError(String),
}

impl From<tokio::task::JoinError> for AppError {
    fn from(e: tokio::task::JoinError) -> Self {
        AppError::JoinError(e.to_string())
    }
}

#[derive(Debug)]
pub struct AppState {
    pub doc_store: Arc<Mutex<DocumentStore>>,
    pub embedding_generator: Arc<EmbeddingGenerator>,
    pub conversation: Mutex<Conversation>,
    pub buffer: Mutex<String>,
    pub logger: Arc<Mutex<Logger>>,  
    pub api_key: Mutex<Option<SecretString>>,
    pub preferences: Mutex<Preferences>, 
    pub app_handle: Option<AppHandle>,
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
            doc_store: Arc::new(Mutex::new(doc_store)),
            embedding_generator: Arc::new(embedding_generator),
            conversation: Mutex::new(Conversation::new(32000)),
            buffer: Mutex::new(String::new()),
            api_key: Mutex::new(None),
            preferences: Mutex::new(Preferences::default()), // Start with default preferences
            app_handle: None,
        };
        Ok(app_state)
    }
    
    // pub async fn set_logger_path(&self, path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    //     let path_str = path.to_str().ok_or("Invalid path")?;
    //         // .ok_or("Invalid path")?;
    //     // Create new logger first to ensure it's valid
    //     let new_logger = Logger::new(&path_str)?;
        
    //     // Get lock and replace logger
    //     let mut logger_guard = self.logger.lock().await;
    //     *logger_guard = new_logger;
    //     println!("Logger is now set to: {:?}", logger_guard);
    //     // Verify the change
    //     let current_path = logger_guard.get_logger_path();
    //     if current_path != path {
    //         return Err("Logger path mismatch after setting".into());
    //     }
    
    //     println!("Logger path updated to: {:?}", current_path);
    //     Ok((path_str.to_string()))
    // }
    pub async fn set_logger_path(&self, path: PathBuf) -> Result<(), AppError> {
        let path_str = path.clone().into_os_string().into_string().map_err(|_| AppError::InvalidPath("Invalid UTF-8 path".to_string()))?;
    
        // Create new logger first to ensure it's valid
        let new_logger = task::spawn_blocking(move || {
            let logger_result = Logger::new(&path_str);
            match logger_result {
                Ok(logger) => Ok(logger),
                Err(e) => {
                    eprintln!("Failed to create logger: {}", e);
                    Err(AppError::LoggerCreationError(e.to_string()))
                }
            }
        }).await??;
    
        // Get lock and replace logger
        let mut logger_guard = self.logger.lock().await;
        *logger_guard = new_logger;
    
        println!("Logger path updated to: {:?}", path);
        Ok(())
    }

    pub async fn get_logger_path(&self) -> String {
        let logger = self.logger.lock().await;
        logger.get_logger_path().to_str().unwrap().to_string()
    }
    
    // pub async fn set_logger(&self, logger: Logger) {
    //     *self.logger.lock().await = logger;
    //     println!("Logger set");
    //     println!("Logger path: {:?}", self.get_logger_path().await);
    // }
    
    // // ✅ Load API key from a file
    // pub async fn load_api_key(&self, app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    //     let path = app_handle
    //     .path()
    //     .app_local_data_dir()
    //     .unwrap_or_default()
    //     .join("api_key.txt");
        
    //     if let Ok(contents) = fs::read_to_string(&path) {
    //         let mut api_key = self.api_key.lock().await;
    //         *api_key = Some(contents.trim().to_string());
    //         println!("Loaded API Key: {}", contents.trim());
    //     }
        
    //     Ok(())
    // }


    // pub async fn save_api_key_to_keyring(&self, app_handle: &AppHandle, key: String) -> Result<(), Box<dyn std::error::Error>> {
    //     let entry = Entry::new("openai-ghostwriter", "ghostwriter")
    //     .map_err(|e| {
    //         let error_msg = format!("Failed to create keyring entry: {}", e);
    //         log::error!("{}", error_msg);
    //         e
    //     })?;

    // entry.set_password(&key)
    //     .map_err(|e| {
    //         let error_msg = format!("Failed to save API key to keyring: {}", e);
    //         log::error!("{}", error_msg);
    //         Box::new(e) as Box<dyn std::error::Error>
    //     })?;

    // Ok(())

    // }
    
    // // ✅ Save API key to a file
    // pub async fn save_api_key(&self, _app_handle: &AppHandle, key: String) -> Result<(), Box<dyn std::error::Error>> {
    //     let env_path = ".env"; // ✅ Save to .env in the app's root directory
        
    //     // ✅ Read existing .env contents (if any)
    //     let mut env_contents = fs::read_to_string(env_path).unwrap_or_else(|_| String::new());
        
    //     // ✅ Remove any existing `OPENAI_API_KEY` entry
    //     env_contents = env_contents
    //     .lines()
    //     .filter(|line| !line.starts_with("OPENAI_API_KEY="))
    //     .map(|line| format!("{}\n", line))
    //     .collect();
        
    //     // ✅ Append the new API key entry
    //     env_contents.push_str(&format!("OPENAI_API_KEY={}\n", key));
        
    //     // ✅ Write back to .env
    //     fs::write(env_path, env_contents)?;
        
    //     let mut api_key = self.api_key.lock().await; // ✅ Store it in-memory as well
    //     *api_key = Some(key);
        
    //     println!("API Key saved to .env.");
    //     Ok(())
    // }
}
