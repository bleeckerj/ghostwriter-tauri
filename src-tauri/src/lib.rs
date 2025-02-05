#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use epub::doc;
use std::io::Stdout;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{generate_handler, Builder};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri::{AppHandle, Manager};

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
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

        // Load .env file
    
}
