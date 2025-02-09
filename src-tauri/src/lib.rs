#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use epub::doc;
use std::io::Stdout;
use std::path::PathBuf;
use std::sync::Mutex;
use serde::Serialize;
use serde_json::Value;

use tauri::{generate_handler, Builder, Emitter, AppHandle, Manager};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

mod document_store;
mod embeddings;

use chrono::Utc;
use document_store::DocumentStore;
use embeddings::EmbeddingGenerator;

mod conversations; // Add this line
use conversations::Conversation;

mod app_state; // Add this line
use app_state::AppState;
use tauri_plugin_log::{Target, TargetKind};

use async_openai::{
    //config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
// mod logger;
// use logger::{CompletionLogEntry, Logger, VectorSearchResult};
use std::env;
use lazy_static::lazy_static;

lazy_static! {
    static ref OPENAI_API_KEY: String = {
        dotenv::dotenv().ok();
        env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set")
    };
}



#[tauri::command]
async fn completion_from_context(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    input: String,
) -> Result<String, String> {
    
    //println!("Processing completion request. `{}`", truncate(&input, 20));
    Ok("Hello".to_string())
}


#[tauri::command]
async fn test_log_emissions(
    state: tauri::State<'_, AppState>,
    logger: tauri::State<'_, NewLogger>,
    app_handle: tauri::AppHandle,
    message: String,
) -> Result<String, String> {
    // Step 1: Create rich log
    let rich_log_data = RichLog {
        message:message.to_string(),
        data: message.clone(),
        timestamp: chrono::Local::now().to_rfc3339(),
        level: "info".to_string(),
    };
    let simple_log_data = SimpleLog {
        message: format!("Processing completion request. Input: `{}`", message),
        timestamp: chrono::Local::now().to_rfc3339(),
        level: "error".to_string(),
    };
    logger.simple_log_message(
        format!("{}", message),
        "error".to_string()
    );    // app_handle.emit("simple-log-message", simple_log_data).unwrap();
    // app_handle.emit("rich-log-message", rich_log_data).unwrap();
    Ok("Logged".to_string())
}


#[tauri::command]
async fn greet(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    name: &str
) -> Result<String, String> {
    // Call completion_from_context with received parameters
    //completion_from_context(state, app_handle, name.to_string()).await?;

    let message;
    if name.is_empty() {
        message = "Hello".to_string();
    } else {
        message = format!("Hello Hello, {}! You've been greeted from Rust!", name);    
    }
    Ok(message)
    
}

#[tauri::command]
async fn simple_log_message(
    logger: tauri::State<'_, NewLogger>,
    message: String,
    level: String,
) -> Result<String, String> {
    logger.simple_log_message(message, level);
    Ok("Simple Logged".to_string())
}

#[tauri::command]
async fn rich_log_message(
    logger: tauri::State<'_, NewLogger>,
    message: String,
    data: String,
    level: String,
) -> Result<String, String> {
    logger.rich_log_message(message, data, level);
    Ok("Rich Logged".to_string())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    
    match dotenv::dotenv() {
        Ok(_) => println!("Successfully loaded .env file"),
        Err(e) => eprintln!("Error loading .env file: {}", e),
    }
    let some_variable = std::env::var("OPENAI_API_KEY").expect("SOME_VARIABLE not set");
    
    let embedding_generator = EmbeddingGenerator::new();
    let path = PathBuf::from("./resources/ghostwriter-selectric/vector_store/");
    
    println!("Initializing DocumentStore with path: {:?}", path);

    let doc_store = DocumentStore::new(path.clone()).expect(&format!(
        "Failed to initialize document store at path: {:?}",
        path
    ));
    println!("DocumentStore successfully initialized.");
    let embedding_generator = EmbeddingGenerator::new();
    let app_state = AppState::new(
        doc_store,
        embedding_generator,
        "path/to/log.txt"
    ).expect("Failed to create AppState");


    tauri::Builder::default()
    .manage(app_state)
    .setup(|app| {
        let app_handle = app.handle();
        let new_logger = NewLogger::new(app_handle.clone());
        new_logger.simple_log_message(
            "Ghostwriter Up.".to_string(),
            "info".to_string()
        );
        new_logger.rich_log_message(
            "Ghostwriter Up.".to_string(),
            "Ghostwriter is up and running.".to_string(),
            "info".to_string()
        );
        app.manage(new_logger.clone());
        // Load .env file
        dotenv::dotenv().ok();
        Ok(())
    })
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![
        greet,
        completion_from_context,
        test_log_emissions,
        simple_log_message,
        rich_log_message,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
    

    
}

#[derive(Serialize, Clone)]
struct RichLog {
    message: String,
    data: String,
    timestamp: String,
    level: String,
}

#[derive(Serialize, Clone)]
struct SimpleLog {
    message: String,
    timestamp: String,
    level: String,
}


fn truncate(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        None => s.to_string(),
        Some((idx, _)) => format!("{}...", &s[..idx])
    }
}

#[derive(Clone)]
struct NewLogger {
    app_handle: AppHandle,
}

impl NewLogger {
    fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    fn simple_log_message(&self, message: String, level: String) {
        let simple_log_data = SimpleLog {
            message: format!("{}", message),
            level: level.clone(),
            timestamp: chrono::Local::now().to_rfc3339(),
        };
        self.app_handle.emit("simple-log-message", simple_log_data).unwrap();
    }

    fn rich_log_message(&self, message: String, data: String, level: String) {
        let rich_log_data = RichLog {
            message: message.clone(),
            data: data.clone(),
            timestamp: chrono::Local::now().to_rfc3339(),
            level: level.clone(),
        };
        self.app_handle.emit("rich-log-message", rich_log_data).unwrap();
    }
}