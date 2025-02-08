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
mod logger;
use logger::{CompletionLogEntry, Logger, VectorSearchResult};
use std::env;
use lazy_static::lazy_static;

lazy_static! {
    static ref OPENAI_API_KEY: String = {
        dotenv::dotenv().ok();
        env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set")
    };
}

#[derive(Serialize, Clone)]
struct RichLog {
    message: String,
    data: String,
    timestamp: String,
}

#[derive(Serialize, Clone)]
struct SimpleLog {
    message: String,
    timestamp: String,
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
    app_handle: tauri::AppHandle,
    input: String,
) -> Result<String, String> {
    // Step 1: Create rich log
    let rich_log_data = RichLog {
        message: "This might be a piece of the canon".to_string(),
        data: input.clone(),
        timestamp: chrono::Local::now().to_rfc3339(),
    };
    let simple_log_data = SimpleLog {
        message: format!("Processing completion request. Input: `{}`", input),
        timestamp: chrono::Local::now().to_rfc3339(),
    };
    app_handle.emit("simple-log-message", simple_log_data).unwrap();
    app_handle.emit("rich-log-message", rich_log_data).unwrap();
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
        message = format!("Hello! You've been greeted from Rust!")
    } else {
        message = format!("Hello, `{}`! You've been greeted from Rust!", name)
    }
    Ok(message)
    
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
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![
        greet,
        completion_from_context,
        test_log_emissions,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
    
    // Load .env file
    
}


fn truncate(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        None => s.to_string(),
        Some((idx, _)) => format!("{}...", &s[..idx])
    }
}