#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use ai::ModelProvider;
use epub::doc;
use futures::FutureExt;
use pdf_extract::Path;
use std::f32::consts::E;
use std::io::Stdout;
use std::path::PathBuf;
use std::sync::Mutex;
use std::fmt;
use std::collections::HashSet;
use serde::Serialize;
use serde_json::Value;
use tokio::time::{sleep, Duration};
use tauri::{generate_handler, Runtime, Builder, Emitter, AppHandle, Manager, Window, State, WebviewWindowBuilder, WebviewWindow, WebviewUrl};
use chrono::{Local, Utc};  // Add Utc here
use std::sync::Arc;
use rand::seq::SliceRandom; 
use tauri_plugin_log::{Target, TargetKind};
extern crate log;
use syslog::{Facility, Formatter3164, BasicLogger};
use log::{SetLoggerError, LevelFilter, info};
use serde_json::json;
use tauri::path::{BaseDirectory};
mod preferences;
use preferences::Preferences;
mod keychain_handler;
use keychain_handler::KeychainHandler;

use crate::ai::AIProviderError;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use window_vibrancy::{apply_blur, apply_vibrancy, NSVisualEffectMaterial};
use embeddings::EmbeddingGenerator;
use document_store::DocumentStore;

pub mod ai;
pub mod ingest;
pub mod document_store;
pub mod menu;
pub mod embeddings;

mod conversations; // Add this line
use conversations::Conversation;

mod app_state; // Add this line
use app_state::AppState;
use crate::ai::providers::{self, ProviderType, Provider};
use crate::ai::models::{ChatCompletionRequest, ChatMessage, MessageRole, EmbeddingRequest};
use crate::ai::traits::{EmbeddingProvider, ChatCompletionProvider, PreferredEmbeddingModel};

// Define log levels as constants
pub const LOG_INFO: &str = "info";
pub const LOG_DEBUG: &str = "debug";
pub const LOG_ERROR: &str = "error";
pub const LOG_WARN: &str = "warn";


/// Logs a message using the application's logging system and returns the message string.
///
/// This macro creates a structured log entry, sends it to the frontend via event emission,
/// and returns the formatted message for further use.
///
/// # Parameters
///
/// The macro supports three different calling patterns:
///
/// ## Basic version (with default INFO level):
/// - `$app_handle`: A reference to the tauri::AppHandle
/// - `$fmt`: A format string literal, similar to format!()
/// - `$($arg)*`: Format arguments
///
/// ## With custom log level:
/// - `$app_handle`: A reference to the tauri::AppHandle
/// - `$level`: Log level constant (LOG_INFO, LOG_DEBUG, LOG_ERROR, LOG_WARN)
/// - `$fmt`: A format string literal
/// - `$($arg)*`: Format arguments
///
/// ## With custom log level and ID:
/// - `$app_handle`: A reference to the tauri::AppHandle
/// - `$level`: Log level constant (LOG_INFO, LOG_DEBUG, LOG_ERROR, LOG_WARN)
/// - `$id`: String identifier for the log message (for grouping/filtering)
/// - `$fmt`: A format string literal
/// - `$($arg)*`: Format arguments
///
/// # Returns
///
/// Returns the formatted message string
///
/// # Examples
///
/// ```rust
/// // Basic usage (INFO level)
/// let msg = log_message!(app_handle, "Processing file: {}", filename);
///
/// // With custom log level
/// let error = log_message!(app_handle, LOG_ERROR, "Failed to process: {}", e);
/// return Err(error);
///
/// // With ID for grouping related logs
/// log_message!(app_handle, LOG_WARN, "file-processing", "Issue with file {}: {}", filename, issue);
/// ```
#[macro_export]
macro_rules! log_message {
    // Basic version: message, level
    ($app_handle:expr, $fmt:literal, $($arg:tt)*) => {{
        let message = format!($fmt, $($arg)*);
        let logger = NewLogger::new($app_handle.clone());
        logger.simple_log_message(message.clone(), "".to_string(), LOG_INFO.to_string());
        message
    }};
    
    ($app_handle:expr, $level:expr, $fmt:literal) => {{
        let message = format!($fmt);
        let logger = NewLogger::new($app_handle.clone());
        logger.simple_log_message(message.clone(), "".to_string(), $level.to_string());
        message
    }};
    
    // Version with custom level - specifying $fmt must be a literal
    ($app_handle:expr, $level:expr, $fmt:literal, $($arg:tt)*) => {{
        let message = format!($fmt, $($arg)*);
        let logger = NewLogger::new($app_handle.clone());
        logger.simple_log_message(message.clone(), "".to_string(), $level.to_string());
        message
    }};
    
    // Version with level and ID
    ($app_handle:expr, $level:expr, $id:expr, $fmt:literal, $($arg:tt)*) => {{
        let message = format!($fmt, $($arg)*);
        let logger = NewLogger::new($app_handle.clone());
        logger.simple_log_message(message.clone(), $id.to_string(), $level.to_string());
        message
    }};
}

use async_openai::{
    //config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client, config::OpenAIConfig,
};
mod logger;
use logger::{Completion, CompletionLogEntry, Logger, VectorSearchResult};
use std::env;
use lazy_static::lazy_static;
use std::time::Instant;

lazy_static! {
    static ref OPENAI_API_KEY: Option<String> = {
        dotenv::dotenv().ok();
        env::var("OPENAI_API_KEY").ok()
    };
    static ref PDF_LIB_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
    static ref RESOURCE_DIR_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
    static ref API_KEY_MISSING: Mutex<bool> = Mutex::new(false);
}

// At the top of your file with other constants
const MENU_FILE_NEW: &str = "file-new";
const MENU_FILE_QUIT: &str = "file-quit";

pub fn get_pdf_lib_path() -> Option<PathBuf> {
    PDF_LIB_PATH.lock().unwrap().clone()
}

pub fn get_resource_dir_path() -> Option<PathBuf> {
    RESOURCE_DIR_PATH.lock().unwrap().clone()
}


pub fn get_preferred_llm_provider(app_handle: &AppHandle, preferences: &Preferences) -> Result<Provider, String> {
    // Create provider based on preferences
    let mut new_logger = NewLogger::new(app_handle.clone());
    let provider = match preferences.ai_provider.to_lowercase().as_str() {
        "ollama" => {
            new_logger.simple_log_message(
                format!("Using Ollama provider at: {}", preferences.ollama_url),
                "provider".to_string(),
                "info".to_string()
            );
            providers::create_provider(ProviderType::Ollama, &preferences.ollama_url)
        },
        "lmstudio" => {
            new_logger.simple_log_message(
                format!("Using LM Studio provider at: {}", preferences.lm_studio_url),
                "provider".to_string(),
                "info".to_string()
            );
            providers::create_provider(ProviderType::LMStudio, &preferences.lm_studio_url)
        },
        "openai" | _ => {
            // Default to OpenAI if unrecognized
            let openai_api_key = get_api_key(&app_handle).map_err(|e| e.to_string())?;
            match openai_api_key {
                Some(key) => {
                    new_logger.simple_log_message(
                        "Using OpenAI provider".to_string(),
                        "provider".to_string(),
                        "info".to_string()
                    );
                    providers::create_provider(ProviderType::OpenAI, &key)
                },
                None => {
                    log::warn!("OpenAI API key not found. Cannot use OpenAI provider.");
                    return Err("OpenAI API key is required but was not found. Check preferences and/or system keychain.".to_string());
                }
            }
        }
    };
    Ok(provider)
}

#[derive(Serialize)]
struct CompletionTiming {
    embedding_generation_ms: u128,
    similarity_search_ms: u128,
    llm_request_time_ms: u128,
    total_ms: u128,
}

impl fmt::Display for CompletionTiming {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Completion Timing:\n\
            Embedding Generation: {} ms\n\
            Similarity Search: {} ms\n\
            LLM Inference Request: {} ms\n\
            Total: {} ms",
            self.embedding_generation_ms,
            self.similarity_search_ms,
            self.llm_request_time_ms,
            self.total_ms
        )
    }
}

#[tauri::command]
fn turn_on_vibrancy(app_handle: AppHandle, window_label: String) -> Result<(), String> {
    let window = app_handle.get_webview_window(&window_label)
        .ok_or_else(|| format!("Window with label '{}' not found", window_label))?;
    
    #[cfg(target_os = "macos")]
    apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, Some(10.0))
        .map_err(|e| format!("Failed to apply vibrancy: {}", e))?;
    
    #[cfg(target_os = "windows")]
    apply_blur(&window, Some((18, 18, 18, 125)))
        .map_err(|e| format!("Failed to apply blur: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn get_model_names(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    provider_name: String,
) -> Result<Vec<String>, String> {
    let preferences = state.preferences.lock().await;
    //let provider = get_preferred_llm_provider(&app_handle, &preferences).map_err(|e| e.to_string())?;
    let provider = match provider_name.to_lowercase().as_str() {
        "ollama" => {
            let ollama_url = preferences.ollama_url.clone();
            providers::create_provider(ProviderType::Ollama, &ollama_url)
        },
        "lmstudio" => {
            let lmstudio_url = preferences.lm_studio_url.clone();
            providers::create_provider(ProviderType::LMStudio, &lmstudio_url)
        },
        "openai" => {
            let openai_api_key = get_api_key(&app_handle).map_err(|e| e.to_string())?;
            match openai_api_key {
                Some(key) => providers::create_provider(ProviderType::OpenAI, &key),
                None => {
                    log::warn!("OpenAI API key not found. Cannot use OpenAI provider.");
                    return Err("OpenAI API key is required but was not found. Check preferences and/or system keychain.".to_string());
                }
            }
        },
        | _ => {
            log::warn!("Unknown provider name: {}", provider_name);
            return Err(format!("Unknown provider name {}", provider_name));
        }
    };
    let new_logger = NewLogger::new(app_handle.clone());
    let models = match provider.list_models().await {
        Ok(models) => models,
        Err(e) => {
            let error_message = format!("Failed to list models: {}", e);
            log::error!("{}", error_message); // Log the error
            new_logger.simple_log_message(error_message.clone(), "models".to_string(), "error".to_string());
            Vec::new() // Return an empty list of models
        }
    };
    models.iter().for_each(|model| {
        log::debug!("Model: {:?}", model);
        new_logger.simple_log_message(
            format!("Model: {:?}", model),
            "models".to_string(),
            "debug".to_string()
        );
    });
    let model_names: Vec<String> = models.iter().map(|model| model.name.clone()).collect();
    Ok(model_names)
}

#[tauri::command]
async fn ingest_from_url(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    url: String,
) -> Result<(), String> {
    
    let store = state.doc_store.lock().await;
    let store_clone = Arc::new(store.clone());
    let preferences = state.preferences.lock().await;
    
    let provider = match get_preferred_llm_provider(&app_handle, &preferences) {
        Ok(p) => p,
        Err(e) => {
            let line = line!();
            log_message!(app_handle, LOG_ERROR, "Line {} - Provider initialization failed: {}", line, e);
            return Err(format!("Line {} — Could not initialize AI provider: {}", line, e));
        }
    };
    
    match store_clone.ingest_url_async(&url, &provider, app_handle.clone()).await {
        Ok(ingested_document) => {
            log::info!("Ingested URL: {}", url);
            log_message!(app_handle, LOG_INFO, "Ingested URL: {}", url);
            
            // Get current date/time and format it as mmddyy_hhmmss
            let now = chrono::Local::now();
            let date_time_str = now.format("%m%d%y_%H%M%S").to_string();
            
            // Create the filename with date_time prefix and then sanitize
            let title_with_date = format!("{}_{}", &ingested_document.title, date_time_str);
            let suggested_filename = sanitize_filename(&title_with_date);
            
            // Use tauri's dialog to show a save dialog
            app_handle.dialog()
            .file()
            .add_filter("Markdown", &["md", "mdx"])
            .set_file_name(&suggested_filename)  // Set suggested filename
            .save_file(move |file_path| {
                if let Some(path) = file_path {
                    // User selected a path - now save the file
                    let content = format!(
                        "---\ntitle: {}\nurl: {}\ncreated_date: {}\n---\n\n{}",
                        ingested_document.title,
                        ingested_document.metadata.source_path,
                        ingested_document.metadata.created_date.unwrap_or_default(),
                        ingested_document.content
                    );
                    log_message!(app_handle, LOG_INFO, "Saving document to {}", path.to_string());
                    // Since we're in a closure, we need to spawn a task to do the async write
                    let app_handle_clone = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        match tokio::fs::write(path.as_path().unwrap(), content).await {
                            Ok(_) => {
                                log_message!(app_handle_clone, LOG_INFO, "Saved document to {}", path.to_string());
                            },
                            Err(e) => {
                                log_message!(app_handle_clone, LOG_ERROR, "Failed to save document: {}", e);
                            }
                        }
                    });
                } else {
                    // User cancelled the dialog
                    log_message!(app_handle, LOG_INFO, "Save cancelled by user");
                }
            });
            Ok(())
        },
        Err(e) => {
            log_message!(app_handle, LOG_ERROR, "Failed to ingest url: {}", e);
            log::error!("Failed to ingest url: {}", e);
            Err(format!("Failed to ingest url: {}", e))
        }
    }
}

// Helper function to create a valid filename
fn sanitize_filename(filename: &str) -> String {
    // Replace invalid filename characters
    let filename = filename.replace(&['/','\\',':','*','?','"','<','>','|'][..], "_");
    
    // Trim and add extension if needed
    let filename = filename.trim();
    if !filename.ends_with(".md") {
        format!("{}.md", filename)
    } else {
        filename.to_string()
    }
}

#[tauri::command]
async fn save_text_content(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    file_path: String,
    content: String,
) -> Result<(), String> {
    let path = PathBuf::from(file_path);
    let result = tokio::fs::write(&path, content).await;
    match result {
        Ok(_) => {
            log::debug!("Saved to file: {}", path.to_string_lossy());
            // let message = format!("Saved to file: {}", path.to_string_lossy());
            // let new_logger = NewLogger::new(app_handle.clone());
            // new_logger.simple_log_message(message.clone(), "".to_string(), "info".to_string());
            log_message!(app_handle, LOG_INFO, "Saved to file: {}", path.to_string_lossy());
            Ok(())
        }
        Err(e) => {
            let message = format!("Failed to save to file: {}", e);
            // let new_logger = NewLogger::new(app_handle.clone());
            // new_logger.simple_log_message(message.clone(), "".to_string(), "error".to_string());
            log_message!(app_handle, LOG_ERROR, "Failed to save to file: {}", e);
            Err(message)
        }
    }
}

#[tauri::command]
async fn save_json_content(
    app_handle: tauri::AppHandle,
    file_path: String, 
    content: Value  // Use serde_json::Value to receive any valid JSON
) -> Result<(), String> {
    let path = PathBuf::from(file_path);
    
    // Convert the JSON to a pretty-printed string
    let json_string = match serde_json::to_string_pretty(&content) {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to format JSON: {}", e))
    };
    log::debug!("JSON content: {}", json_string);
    log::debug!("Trying to save JSON content to file: {}", path.to_string_lossy());
    // Write to file
    match tokio::fs::write(&path, json_string).await {
        Ok(_) => {
            log::debug!("Saved JSON content to file: {}", path.to_string_lossy());
            let message = format!("Saved JSON content to file: {}", path.to_string_lossy());
            let new_logger = NewLogger::new(app_handle.clone());
            new_logger.simple_log_message(
                message,
                "file_save".to_string(),
                "info".to_string()
            );
            Ok(())
        },
        Err(e) => {
            let error_msg = format!("Failed to write to file: {}", e);
            log::error!("{}", error_msg);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn load_openai_api_key_from_keyring(
    app_handle: tauri::AppHandle, 
    state: tauri::State<'_, AppState>) -> Result<(String), String> {
        // Create logger instance
        let new_logger = NewLogger::new(app_handle.clone());
        
        // Returns the key if there was a key there (even if it was invalid)
        // Returns error string if there was no key there
        // Returns an error if there was a problem loading the key
        match KeychainHandler::retrieve_api_key() {
            Ok(key) => {
                match key {
                    Some(k) => {
                        log::debug!("API key successfully loaded from keychain");
                        new_logger.simple_log_message(
                            format!("{} API key successfully loaded from keychain", k),
                            "keychain".to_string(),
                            "debug".to_string()
                        );
                        Ok(k)
                    }
                    None => {
                        log::warn!("No API key found in keychain");
                        new_logger.simple_log_message(
                            "No API key not found in keychain".to_string(),
                            "keychain".to_string(),
                            "warn".to_string()
                        );
                        Ok("Enter API key".to_string())
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Keychain issue? Failed to load API key from keychain: {}", e);
                log::error!("{}", error_msg);
                new_logger.simple_log_message(
                    error_msg.clone(),
                    "keychain".to_string(),
                    "error".to_string()
                );
                Err(error_msg)
            }
        }
        
        
    }
    
    
    #[tauri::command]
    async fn save_openai_api_key_to_keyring(
        app_handle: tauri::AppHandle, 
        state: tauri::State<'_, AppState>, 
        key: String
    ) -> Result<(), String> {
        // Create logger instance
        let new_logger = NewLogger::new(app_handle.clone());
        
        // Attempt to store the key
        match KeychainHandler::store_api_key(&key) {
            Ok(_) => {
                log::info!("API key successfully stored in keychain");
                new_logger.simple_log_message(
                    "API key successfully stored in keychain".to_string(),
                    "keychain".to_string(),
                    "debug".to_string()
                );
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to store API key in keychain: {}", e);
                log::error!("{}", error_msg);
                new_logger.simple_log_message(
                    error_msg.clone(),
                    "keychain".to_string(),
                    "error".to_string()
                );
                Err(error_msg)
            }
        }
    }
    
    /**
    *  PREFERENCES
    */
    #[tauri::command]
    async fn get_preferences(state: tauri::State<'_, AppState>) -> Result<Preferences, String> {
        let preferences = state.preferences.lock().await;
        Ok(preferences.clone()) // ✅ Send preferences to frontend
    }
    
    
    #[tauri::command]
    async fn load_preferences(app_handle: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(Preferences), String> {
        let preferences = Preferences::load_with_defaults(&state, app_handle.clone());
        *state.preferences.lock().await = preferences.clone();
        Ok((preferences))
    }

    /**
     * Canon list view control panel thing
     */
    #[tauri::command]
    async fn open_canon_control_panel(app_handle: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
        let preferences = state.preferences.lock().await;
        let provider = get_preferred_llm_provider(&app_handle, &preferences).map_err(|e| format!("Couldn't get preferred LLM provider: {}", e))?;
        let store = state.doc_store.lock().await;

        let _ = tauri::WebviewWindowBuilder::new(
            &app_handle,
            "canon-control-panel", // window label
            tauri::WebviewUrl::App("canon-view.html".into()), // path in /dist
          )
          .title("Control Panel")
          .resizable(true)
          .inner_size(600.0, 400.0)
          .always_on_top(false)
          .decorations(false)
          .transparent(true)
          .focused(true)
          .skip_taskbar(false)
          .build();


        Ok(())
    }

    #[tauri::command]
    async fn close_canon_control_panel(app_handle: tauri::AppHandle) -> Result<(), String> {
        let window = app_handle.get_webview_window("canon-control-panel");
        if let Some(window) = window {
            window.close().map_err(|e| format!("Failed to close window: {}", e))?;
        }
        Ok(())
    }

#[tauri::command]
async fn toggle_canon_control_panel(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("control-panel") {
        // If the window exists, close it
        window.close().map_err(|e| format!("Failed to close control panel: {}", e))?;
    } else {
        // If the window does not exist, open it
        let _ = tauri::WebviewWindowBuilder::new(
            &app_handle,
            "canon-control-panel", // window label
            tauri::WebviewUrl::App("canon-view.html".into()), // path in /dist
        )
        .title("Control Panel")
        .resizable(true)
        .inner_size(600.0, 400.0)
        .always_on_top(false)
        .decorations(false)
        .transparent(true)
        .focused(true)
        .skip_taskbar(false)
        .min_inner_size(400.0, 300.0)
        .build();
    }
    Ok(())
}

    #[tauri::command]
    async fn update_preferences(
        app_handle: tauri::AppHandle,
        state: tauri::State<'_, AppState>,
        responselimit: String,
        mainprompt: String,
        finalpreamble: String,
        prosestyle: String,
        similaritythreshold: String,
        shufflesimilars: bool,
        similaritycount: String,
        maxhistory: String,
        maxtokens: String,
        temperature: String,
        gametimerms: String,
        aiprovider: String,
        aimodelname: String,
        ollamaurl: String,
        lmstudiourl: String,
    ) -> Result<(Preferences), String> {
        
        // println!("update_preferences called with: {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
        // responselimit, mainprompt, finalpreamble, prosestyle, similaritythreshold, shufflesimilars, similaritycount, maxhistory, maxtokens, temperature, gametimerms);
        
        let mut preferences = state.preferences.lock().await;
        preferences.response_limit = responselimit;
        preferences.main_prompt = mainprompt;
        preferences.final_preamble = finalpreamble;
        preferences.prose_style = prosestyle;
        preferences.similarity_threshold = similaritythreshold.parse::<f32>().unwrap() / 100.0;
        preferences.shuffle_similars = shufflesimilars == true;
        preferences.similarity_count = similaritycount.parse::<usize>().unwrap_or(Preferences::SIMILARITY_COUNT_DEFAULT);
        preferences.max_history = maxhistory.parse::<usize>().unwrap_or(Preferences::MAX_HISTORY_DEFAULT);
        preferences.max_output_tokens = maxtokens.parse::<usize>().unwrap_or(Preferences::MAX_OUTPUT_TOKENS_DEFAULT);
        preferences.temperature = temperature.parse::<f32>().unwrap_or(Preferences::TEMPERATURE_DEFAULT);
        preferences.game_timer_ms = 1000*(gametimerms.parse::<usize>().unwrap_or(Preferences::GAME_TIMER_MS_DEFAULT));
        preferences.ai_provider = aiprovider;
        preferences.ai_model_name = aimodelname;
        preferences.ollama_url = ollamaurl;
        preferences.lm_studio_url = lmstudiourl;
        
        let prefs_clone = preferences.clone();
        // Attempt to save preferences and handle any errors
        if let Err(e) = preferences.save() {
            let error_message = format!("Failed to save preferences: {}", e);
            let new_logger = NewLogger::new(app_handle.clone());
            new_logger.simple_log_message(error_message.clone(), "preferences".to_string(), "error".to_string());
            return Err(error_message);
        }
        let debug_message = format!("Preferences updated: {:?}", preferences);
        let new_logger = NewLogger::new(app_handle.clone());
        new_logger.simple_log_message(debug_message.clone(), "preferences".to_string(), "debug".to_string());
        Ok((prefs_clone))
    }
    
    #[tauri::command]
    async fn reset_preferences(state: tauri::State<'_, AppState>) -> Result<(Preferences), String> {
        let mut preferences = state.preferences.lock().await;
        preferences.reset_to_defaults();
        preferences.save().map_err(|e| e.to_string()); 
        Ok(preferences.clone())
    }
    
    #[tauri::command]
    async fn prefs_file_path() -> Result<String, String> {
        // let mut preferences = state.preferences.lock().await;
        // *preferences = Preferences::default();
        // let path = preferences.get_preferences_file_path().map_err(|e| e.to_string());
        Ok(Preferences::prefs_file_path())
    }
    
    #[tauri::command]
    async fn get_logger_path(state: tauri::State<'_, AppState>) -> Result<String, String> {
        let logger = state.logger.lock().await;
        let logger_path = logger.get_logger_path().to_str().unwrap().to_string();
        log::debug!("Logger path: {:?}", logger_path);
        Ok(logger_path)
    }
    
    #[tauri::command]
    async fn set_logger_app_data_path(
        state: tauri::State<'_, AppState>,
        app_handle: tauri::AppHandle,
    ) -> Result<(), String> {
        let mut app_data_path: PathBuf = app_handle.path()
        .app_log_dir()
        .unwrap_or_else(|_| {
            // Provide a default path if app_data_dir() returns None
            // This could be a fallback path in the user's home directory or a temporary directory
            std::env::temp_dir()
        });
        app_data_path.push("ghostwriter_log.json");
        
        /// PROBLEMATIC
        state.set_logger_path(app_data_path).await.map_err(|e| e.to_string());
        Ok(())
        
    }
    
    #[tauri::command]
    async fn get_log_contents(state: tauri::State<'_, AppState>) -> Result<Vec<Completion>, String> {
        // Get the logger path from state
        let logger = state.logger.lock().await;
        let log_path = logger.get_logger_path();
        
        // Read the file contents
        let file = std::fs::File::open(log_path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;
        
        // Parse JSON contents
        let contents: Vec<Completion> = serde_json::from_reader(file)
        .map_err(|e| format!("Failed to parse log contents: {}", e))?;
        
        Ok(contents)
    }
    
    #[tauri::command]
    async fn ingestion_from_file_dialog(
        state: tauri::State<'_, AppState>,
        app_handle: tauri::AppHandle,
        file_path: String,
    ) -> Result<String, String> {
        
        //println!("Ingesting file: {}", file_path);
        log::info!("Ingesting file: {}", file_path);
        
        let file_path_buf = PathBuf::from(file_path);
        let file_name = file_path_buf.clone().as_path().file_name().unwrap().to_str().unwrap().to_string();
        
        let preferences = state.preferences.lock().await;
        let provider = get_preferred_llm_provider(&app_handle, &preferences)
        .map_err(|e| format!("Couldn't get a preferred LLM provider: {}", e))?;        let store = state.doc_store.lock().await;
        let store_clone = Arc::new(store.clone());
        store_clone.process_document_async(&provider, &file_path_buf, app_handle).await;
        
        Ok("Ingested file".to_string())
    }
    
    
    // Modify the function return type to include timing
    #[tauri::command]
    async fn completion_from_context(
        state: tauri::State<'_, AppState>,
        app_handle: tauri::AppHandle,
        input: String,
    ) -> Result<(String, CompletionTiming, Completion), String> {
        
        let preferences = state.preferences.lock().await;
        let max_tokens: usize = preferences.max_output_tokens;
        let temperature = preferences.temperature;
        let shuffle_similars = preferences.shuffle_similars;
        let similarity_count = preferences.similarity_count;
        let max_history = preferences.max_history;
        let similarity_threshold = preferences.similarity_threshold;
        let mut new_logger = NewLogger::new(app_handle.clone());
        
        //let openai_api_key = get_api_key(&app_handle).map_err(|e| e.to_string())?;
        //let key_clone = openai_api_key.clone().unwrap();
        let logger_clone = state.logger.clone();
        


        let mut provider = get_preferred_llm_provider(&app_handle, &preferences).map_err(|e| format!("Couldn't get preferred LLM provider: {}", e))?;
        provider.set_preferred_inference_model(preferences.ai_model_name.clone());
        let lm_models = provider.list_models().await;
        lm_models.iter().for_each(|models| {
            models.iter().for_each(|model| {
                log::debug!("Model: {:?}", model.name);
                new_logger.simple_log_message(
                    format!("Model: {:?}", model.name),
                    "models".to_string(),
                    "debug".to_string()
                );
            });
        });
        
        let ai_model_name = preferences.ai_model_name.clone();

        let start_total = Instant::now();
        
        // Time embedding generation
        let start_embedding = Instant::now();
        
        // Log which provider is being used
        match &provider {
            Provider::OpenAI(_) => {
                log::debug!("Using OpenAI provider for embeddings");
                new_logger.simple_log_message(
                    "Using OpenAI provider for embeddings".to_string(),
                    "embeddings".to_string(),
                    "info".to_string()
                );
            },
            Provider::LMStudio(_) => {
                log::debug!("Using LM Studio provider for embeddings");
                new_logger.simple_log_message(
                    "Using LM Studio provider for embeddings".to_string(),
                    "embeddings".to_string(),
                    "info".to_string()
                );
            },
            Provider::Ollama(_) => {
                log::debug!("Using Ollama provider for embeddings");
                new_logger.simple_log_message(
                    "Using Ollama provider for embeddings".to_string(),
                    "embeddings".to_string(),
                    "info".to_string()
                );
            }
        };
        
        // Create the embedding request
        let embedding_request = EmbeddingRequest {
            model: provider.get_preferred_embedding_model(),
            input: vec![input.clone()],
        };
        
        // Use the provider directly to create embeddings
        let embedding_result = Some(provider.create_embeddings(embedding_request).await);
        
        let embedding_duration = start_embedding.elapsed();
        
        // Handle the case where embedding_result is None
        let embedding_result = match embedding_result {
            Some(result) => result,
            None => {
                return Err("Embedding generation failed: No suitable provider found".to_string());
            }
        };
        
        // Time similarity search
        let start_search = Instant::now();
        let mut similar_docs: Vec<(i64, String, usize, String, f32)> = Vec::new();
        let database_name: String;
        let database_path: String;
        // similar docs get the top 4, which may all be from the same source
        // fence this off so we can release the lock on the store
        {
            let store = state.doc_store.lock().await;
            similar_docs = store.search(&embedding_result, &provider, similarity_count, similarity_threshold).await.map_err(|e| e.to_string())?;
            database_name = (store.get_database_name().to_string()); // Just convert &str to String
            database_path = store.get_database_path().to_string(); // Just convert &str to String
        }
        // lock on store should be released by now
        
        let search_duration = start_search.elapsed();
        
        // Shuffle similar_docs if shuffle_similars is true
        if shuffle_similars {
            new_logger.simple_log_message("Will shuffle similarity docs".to_string(), "".to_string(), "info".to_string());
            let mut rng = rand::thread_rng();
            similar_docs.shuffle(&mut rng);
        }
        
        
        if(similar_docs.len() == 0) {
            new_logger.simple_log_message(
                "No similar documents found. No emanations will issue.".to_string(),
                "".to_string(),
                "info".to_string()
            );
            return Err("No similar documents found. No emanations will issue.".to_string());
        } else {
            new_logger.simple_log_message(
                format!("Found {} cosine similar documents ({})", similar_docs.len(), similarity_threshold),
                "".to_string(),
                "info".to_string()
            );
        }
        // Prepare the context for the LLM
        // This has all the document metadata..is that okay?
        let mut context = String::new();
        for (i, (doc_id, doc_name, chunk_id, chunk_text, similarity)) in similar_docs.iter().enumerate() {
            context.push_str(&format!("{}\n", chunk_text));
        }
        
        let mut vector_search_results_for_log: Vec<VectorSearchResult> = Vec::new();
        let mut seen_chunk_ids: HashSet<usize> = HashSet::new();
        
        for (i, (doc_id, doc_name, chunk_id, chunk_text, similarity)) in similar_docs.iter().enumerate() {
            if seen_chunk_ids.contains(chunk_id) {
                // Skip this item if the chunk_id is already in the HashSet
                continue;
            }
            let msg = format!("<div>
            <div class='border-l-[4px] border-amber-300 pl-2 pr-8 text-pretty leading-tight font-[InputMono]'>{}</div>
            <div class='mt-2 px-2 py-1 rounded-sm bg-gray-700 w-fit'>{}</div>
            <span class='mt-2 font-bold'>{}</span>
          </div>", chunk_text, similarity, doc_name);
            
            new_logger.simple_log_message(msg, chunk_id.to_string(), "info".to_string());
            
            vector_search_results_for_log.push(VectorSearchResult {
                similarity: *similarity,
                name: doc_name.clone(),
                content: chunk_text.clone(),
                chunk_id: *chunk_id,
            });
            // Add the chunk_id to the HashSet
            seen_chunk_ids.insert(*chunk_id);
        }
        
        let conversation_context = state.conversation.lock().await.get_context();
        
        let prose_style = preferences.prose_style.clone();
        
        let response_limit: String = preferences.response_limit.clone();
        
        let main_prompt: String = preferences.main_prompt.clone();
        
        let final_preamble: String = preferences.final_preamble.clone();
        
        let mut system_content = main_prompt.clone();
        system_content.push_str("<response_limit>\nStrictly follow these explicit instructions in terms of quantity and length of your response:\n");
        system_content.push_str(&response_limit);
        system_content.push_str("</response_limit>");
        //system_content.push_str("<previous_exchanges>");
        //system_content.push_str(&conversation_context);
        //system_content.push_str("</previous_exchanges>");
        system_content.push_str("<context>");
        system_content.push_str(&context);
        system_content.push_str("</context>");
        system_content.push_str("<final_preamble>");
        system_content.push_str(&final_preamble);
        system_content.push_str("</final_preamble>");
        system_content.push_str("<prose_style>");
        system_content.push_str(&prose_style);
        system_content.push_str("</prose_style>");
        // system_content.push_str("<user_input>");
        // system_content.push_str(&input);
        // system_content.push_str("</user_input>");


        
    // Create message array in the generic format
    let messages = vec![
    ChatMessage {
        role: MessageRole::System,
        content: system_content.clone(),
        name: None,
    },
    ChatMessage {
        role: MessageRole::User,
        content: input.clone(),
        name: None,
    },
    ];
    
    let provider_chat_model = provider.get_preferred_inference_model(&ai_model_name).await.map_err(|e| e.to_string())?;
    
    // Use the model name from preferences
    let chat_request = ChatCompletionRequest {
        messages,
        model: provider_chat_model.name.clone(),
        temperature: Some(temperature),
        max_tokens: Some(max_tokens as u32),
        stream: false,
    };
    
    // Time AI request
    let start_llm_action = Instant::now();
    
    // Make the request through the provider
    let chat_response = provider
    .create_chat_completion(&chat_request)
    .await
    .map_err(|e| format!("AI completion failed: {}", e))?;
    
    let llm_action_duration = start_llm_action.elapsed();
    let total_duration = start_total.elapsed();
    
    // Process the response
    if let Some(choice) = chat_response.choices.first() {
        let content = &choice.message.content;
        
        // Create timing info
        let timing = CompletionTiming {
            embedding_generation_ms: embedding_duration.as_millis(),
            similarity_search_ms: search_duration.as_millis(),
            llm_request_time_ms: llm_action_duration.as_millis(),
            total_ms: total_duration.as_millis(),
        };
        //let model = provider.get_preferred_inference_model().await.map_err(|e| e.to_string())?;
        let model_name = &provider_chat_model.name;
        let provider_name = provider.get_provider_name();
        let entry = Completion {
            completion: CompletionLogEntry {
                timestamp: Utc::now(),
                completion_result: content.clone(),
                input_text: input.to_string(),
                system_prompt: system_content.clone(),
                conversation_context: conversation_context.clone(),
                vector_search_results_for_log: vector_search_results_for_log,
                canon_name: database_name,
                canon_path: database_path,
                preferences: preferences.clone(),
                llm_provider_name: "Test".to_string(),
                llm_model_name: model_name.to_string(),
                
            }
        };
        
        let new_logger = NewLogger::new(app_handle.clone());
        
        new_logger.simple_log_message(
            timing.to_string(),
            "completion_time".to_string(),
            "info".to_string()
        );
        
        // state
        // .logger
        // .lock()
        // .await
        // .log_completion(entry)
        // .map_err(|e| e.to_string())?;
        // Instead of directly failing if logging fails, log the error and continue
        match state.logger.lock().await.log_completion(entry.clone()) {
            Ok(_) => {
                new_logger.simple_log_message(
                    "Successfully logged completion".to_string(),
                    "completion_log".to_string(),
                    "debug".to_string()
                );
            },
            Err(e) => {
                new_logger.simple_log_message(
                    format!("Failed to log completion: {}", e),
                    "completion_log".to_string(),
                    "error".to_string()
                );
                // Continue execution despite logging failure
            }
        };
        
        let mut conversation = state.conversation.lock().await;
        
        conversation.add_exchange(input.clone(), content.clone(), max_history);
        
        new_logger.simple_log_message(format!("History context is {} exchanges and {} characters", conversation.get_history().len(), conversation.get_context().len()), "".to_string(), "info".to_string());
        //println!("Completion: {}", content);
        return Ok((content.clone(), timing, entry.clone()));
    }
    Err("No completion returned.".to_string())
    
}


fn get_api_key(app_handle: &AppHandle) -> Result<Option<String>, String> {
    match KeychainHandler::retrieve_api_key() {
        Ok(key) => {
            match key {
                Some(k) if k.is_empty() => {
                    // Handle empty string case
                    log::warn!("API key found in keychain but it is empty");
                    let new_logger = NewLogger::new(app_handle.clone());
                    new_logger.simple_log_message(
                        "API key is empty. Please provide a valid key.".to_string(),
                        "keychain".to_string(),
                        "warn".to_string()
                    );
                    Ok(None)  // Treat empty string the same as None
                },
                Some(k) => {
                    log::info!("API key successfully loaded from keychain");
                    // let new_logger = NewLogger::new(Tauri::.clone());
                    // new_logger.simple_log_message(
                    //     format!("{} API key successfully loaded from keychain", k),
                    //     "keychain".to_string(),
                    //     "debug".to_string()
                    // );
                    Ok(Some(k.clone()))
                }
                None => {
                    log::warn!("No API key found in keychain");
                    // new_logger.simple_log_message(
                    //     "No API key not found in keychain".to_string(),
                    //     "keychain".to_string(),
                    //     "warn".to_string()
                    // );
                    Ok(None)
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Keychain issue? Failed to load API key from keychain: {}", e);
            log::error!("{}", error_msg);
            // new_logger.simple_log_message(
            //     error_msg.clone(),
            //     "keychain".to_string(),
            //     "error".to_string()
            // );
            Err(error_msg)
        }
    }
}

// Add this near your other struct definitions
#[derive(Serialize)]
struct SearchResult {
    document_name: String,
    chunk_id: usize,
    chunk_text: String,
    similarity_score: f32,
}

async fn get_current_provider(state: tauri::State<'_, AppState>) -> Result<Provider, String> {
    let state_clone = state.clone();
    let preferences = state.preferences.lock().await;
    let _app_handle = state_clone.app_handle.clone();
    let provider = match preferences.ai_provider.to_lowercase().as_str() {
        "ollama" => {
            let ollama_url = preferences.ollama_url.clone();
            providers::create_provider(ProviderType::Ollama, &ollama_url)
        },
        "lmstudio" => {
            let lmstudio_url = preferences.lm_studio_url.clone();
            providers::create_provider(ProviderType::LMStudio, &lmstudio_url)
        },
        "openai" | _ => {
            let openai_api_key = get_api_key(&_app_handle.ok_or("AppHandle is None")?).map_err(|e| e.to_string())?;
            match openai_api_key {
                Some(key) => {
                    providers::create_provider(ProviderType::OpenAI, &key)
                },
                None => {
                    log::warn!("OpenAI API key not found. Cannot use OpenAI provider.");
                    return Err("OpenAI API key is required but was not found. Check preferences and/or system keychain.".to_string());
                }
            }
        }
    };
    Ok(provider)
}

#[tauri::command]
async fn search_similarity(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    query: String,
    limit: Option<usize>,  
) -> Result<Vec<SearchResult>, String> {  // Changed return type
    let limit = limit.unwrap_or(3);
    let state_clone = state.clone();
    
    let provider = get_current_provider(state.clone()).await?;
    let provider_embedding_model = provider.get_preferred_embedding_model();
    let embedding_request = EmbeddingRequest {
        model: provider_embedding_model,
        input: vec![query.clone()],
    };
    let query_embedding = provider.create_embeddings(embedding_request).await;
    let doc_store = state.doc_store.clone();
    
    // let doc_store = state
    // .doc_store
    // .lock()
    // .map_err(|e| format!("Failed to acquire doc store lock: {}", e))?;
    let preferences = match state_clone.preferences.try_lock() {
        Ok(preferences) => preferences,
        Err(_) => {
            log::error!("Failed to acquire lock on preferences");
            return Err("Failed to acquire lock on preferences".to_string());
        }
    };
    let similarity_threshold = preferences.similarity_threshold;
    let store = state.doc_store.lock().await;
    let results = store
    .search(&query_embedding, &provider, limit, similarity_threshold)
    .await // ✅ Now correctly awaiting the async function
    .map_err(|e| format!("Search failed: {}", e))?;
    
    
    // Transform results into SearchResult structs
    Ok(results
        .into_iter()
        .map(|(doc_id, doc, index, chunk, similarity)| SearchResult {
            document_name: doc,
            chunk_id: index,
            chunk_text: chunk,
            similarity_score: similarity,
        })
        .collect())
    }
    
    #[tauri::command]
    async fn test_log_emissions(
        state: tauri::State<'_, AppState>,
        logger: tauri::State<'_, NewLogger>,
        app_handle: tauri::AppHandle,
        message: String,
    ) -> Result<String, String> {
        println!("Time here looks like {}", chrono::Local::now().to_rfc3339());
        // Step 1: Create rich log
        let rich_log_data = RichLog {
            message:message.to_string(),
            data: message.clone(),
            timestamp: chrono::Local::now().to_rfc3339(),
            level: "info".to_string(),
        };
        let simple_log_data = SimpleLog {
            message: format!("Processing completion request. Input: `{}`", message),
            id: None,
            timestamp: chrono::Local::now().to_rfc3339(),
            level: "error".to_string(),
        };
        logger.simple_log_message(
            format!("{}", message),
            "".to_string(),
            "info".to_string());    // app_handle.emit("simple-log-message", simple_log_data).unwrap();
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
            
            // let progress_indicator = ProgressIndicator {
            //     progress_id: "embedder".to_string(),
            //     current_step: "0".to_string(),
            //     total_steps: "4".to_string(),
            //     current_file: "the-myth".to_string(),
            //     meta: "Ingesting/Embedding".to_string(),
            // };
            
            // load_progress_indicator(&app_handle, progress_indicator);
            
            let message;
            if name.is_empty() {
                message = "Goodbye".to_string();
            } else {
                message = format!("Ciao Ciao, {}!", name);    
            }
            print!("{}", message);
            Ok(message)
            
        }
        
        #[tauri::command]
        async fn simple_log_message(
            logger: tauri::State<'_, NewLogger>,
            message: String,
            id: String,
            level: String,
        ) -> Result<String, String> {
            logger.simple_log_message(message, id, level);
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
        
        #[tauri::command]
        async fn delete_canon_entry(
            logger: tauri::State<'_, NewLogger>,
            app_state: tauri::State<'_, AppState>,
            app_handle: tauri::AppHandle,
            docid: String,
        ) -> Result<String, String> {
            let id = docid.clone();
            
            // Attempt to parse the doc_id string into an i64
            let doc_id_int = match docid.parse::<i64>() {
                Ok(id) => id,
                Err(e) => {
                    let error_message = format!("Failed to parse doc_id as integer: {}", e);
                    logger.simple_log_message(
                        error_message.clone(),
                        id.clone(),
                        "error".to_string(),
                    );
                    return Err(error_message);
                }
            };
            
            let doc_store = Arc::clone(&app_state.doc_store);
            let store = doc_store.lock().await;
            match store.delete_document(doc_id_int).await {
                Ok(_) => {
                    logger.simple_log_message(
                        "Deleted canon entry ".to_string(),
                        id.clone(),
                        "info".to_string(),
                    );
                    Ok("Canon Entry deleted".to_string())
                }
                Err(e) => {
                    let error_message = format!("Failed to delete canon entry: {}", e);
                    logger.simple_log_message(
                        error_message.clone(),
                        id.clone(),
                        "error".to_string(),
                    );
                    Err(error_message)
                }
            }
        }
        
        #[tauri::command]
        async fn list_canon_docs(
            app_state: tauri::State<'_, AppState>,
            app_handle: tauri::AppHandle,
        ) -> Result<String, String> {
            let doc_store = Arc::clone(&app_state.doc_store);
            
            let store = doc_store.lock().await;
            match store.fetch_documents().await {
                Ok(mut listing) => {
                    // Sort the listing by model_name first, then by document name
                    listing.documents.sort_by(|a, b| {
                        // First compare by embedding_model_name
                        let model_cmp = a.embedding_model_name.cmp(&b.embedding_model_name);
                        
                        // If model names are equal, then compare by document name/title
                        if model_cmp == std::cmp::Ordering::Equal {
                            a.name.cmp(&b.name)
                        } else {
                            model_cmp
                        }
                    });
                    
                    let json_string = serde_json::to_string(&listing).map_err(|e| e.to_string())?;
                    app_handle.emit("canon-list", json_string).map_err(|e| e.to_string())?;
                    Ok("Canon list emitted".to_string())
                }
                Err(e) => {
                    let error_message = format!("Failed to fetch canon documents: {}", e);
                    let new_logger = NewLogger::new(app_handle.clone());
                    new_logger.simple_log_message(
                        error_message.clone(),
                        "".to_string(),
                        "error".to_string(),
                    );
                    log::error!("{}", error_message);
                    Err(error_message)
                }
            } 
        }
        
        #[tauri::command]
        async fn toggle_rag_pause(
            app_state: tauri::State<'_, AppState>,
            app_handle: tauri::AppHandle,
            id: String,
            paused: bool,
        ) -> Result<String, String> {
            log::debug!("RAG pause toggled for id: {} to {}", id, paused);
            
            // Parse the document ID
            let doc_id = id.parse::<i64>()
            .map_err(|e| format!("Invalid document ID: {}", e))?;
            
            // Get document store
            let doc_store = Arc::clone(&app_state.doc_store);
            
            // Update the pause state in the database
            let result = doc_store.lock().await.update_document_pause_state(doc_id, paused).await;
            match result {
                Ok(_) => {
                    // Optionally emit an event to notify UI of the state change
                    app_handle.emit("rag-pause-state-changed", json!({
                        "id": doc_id,
                        "paused": paused
                    }))
                    .map_err(|e| format!("Failed to emit event: {}", e))?;
                    
                    Ok(format!("RAG pause toggled to {} for document {}", paused, id))
                },
                Err(e) => {
                    let error_msg = format!("Failed to update pause state: {}", e);
                    log::error!("{}", error_msg);
                    Err(error_msg)
                }
            }
        }
        
        #[tauri::command]
        async fn shot_clock_complete(
            app_state: tauri::State<'_, AppState>,
            app_handle: tauri::AppHandle,
        ) -> Result<String, String> {
            log::debug!("Shot clock complete");
            // what this function does is start the chat completion process    
            Ok("Shot clock complete".to_string())        
        }
        
        #[derive(Serialize, Clone)]
        struct CanonInfo {
            name: String,
            path: String,
        }
        #[tauri::command]
        async fn get_canon_info(
            logger: tauri::State<'_, NewLogger>,
            app_state: tauri::State<'_, AppState>,
            app_handle: tauri::AppHandle
        ) -> Result<CanonInfo, String> {
            
            
            let doc_store = Arc::clone(&app_state.doc_store);
            let store = doc_store.lock().await;
            store.get_database_name();
            store.get_database_path();
            let unlocked_store = store.clone();
            let result = format!("{:?}", unlocked_store);
            
            let canon_info = CanonInfo {
                name: store.get_database_name().to_string(),
                path: store.get_database_path().to_string(),
            };
            
            Ok(canon_info)
        }
        
        
        #[derive(Serialize, Clone)]
        struct ProgressIndicator {
            progress_id: String,
            current_step: String,
            total_steps: String,
            current_file: String,
            meta: String,
        }
        #[derive(Serialize, Clone)]
        struct ProgressUpdate {
            current_step: String,
            current_file: String,
            progress_id: String,
            total_steps: String,
            meta: String,
        }
        
        fn load_progress_indicator(app_handle: &AppHandle, progress_indicator: ProgressIndicator)  
        {
            
            let handle = app_handle.clone();
            
            match app_handle.emit("load-progress-indicator", progress_indicator.clone()) {
                Ok(_) => println!("Progress indicator emitted successfully"),
                Err(e) => {
                    eprintln!("Failed to emit progress indicator: {}", e);
                    let message = format!("Failed to emit progress indicator: {}", e);
                    //new_logger.simple_log_message(message, "".to_string(), "error".to_string());
                },
            }
            // TEST TEST TEST TEST
            // Spawn a test loop that updates progress
            tauri::async_runtime::spawn(async move {
                // Change from 0..=99 to include 100
                for i in 0..=progress_indicator.total_steps.parse::<i32>().unwrap() {
                    let update = ProgressUpdate {
                        current_step: i.to_string(),
                        current_file: progress_indicator.current_file.clone(),
                        progress_id: progress_indicator.progress_id.clone(),
                        total_steps: progress_indicator.total_steps.clone(),
                        meta: progress_indicator.meta.clone(),
                    };
                    
                    match handle.emit("progress-indicator-update", update) {
                        Ok(_) => println!("Progress indicator emitted successfully: step {}", i),
                        Err(e) => eprintln!("Failed to emit progress indicator: {}", e),
                    }
                    
                    // Don't sleep on the final iteration
                    if i < 100 {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            });
            //progress_indicator
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
            id: Option<String>,  // Make id optional
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
            
            fn simple_log_message(&self, message: String, id: String, level: String) {
                let simple_log_data = SimpleLog {
                    message: format!("{}", message),
                    level: level.clone(),
                    timestamp: chrono::Local::now().to_rfc3339().to_string(),
                    id: Some(id.clone()),
                };
                match self.app_handle.emit("simple-log-message", simple_log_data) {
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("Failed to emit simple log: {}", e);
                    }
                }
                //log::debug!("{}", message);
            }
            
            fn rich_log_message(&self, message: String, data: String, level: String) {
                let rich_log_data = RichLog {
                    message: message.clone(),
                    data: data.clone(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    level: level.clone(),
                };
                match self.app_handle.emit("rich-log-message", rich_log_data) {
                    Ok(_) => println!("Rich log emitted successfully"),
                    Err(e) => {
                        eprintln!("Failed to emit rich log: {}", e);
                    }   
                }
                log::info!("{}", message);
            }
        }
        
        
        
        fn check_api_key<R: Runtime>(app_handle: &AppHandle<R>) {
            if *API_KEY_MISSING.lock().unwrap() {
                let _ = WebviewWindowBuilder::new(
                    app_handle,
                    "api_key_window", 
                    WebviewUrl::App("/api_key.html".into()) // ✅ Provide a valid URL
                )
                .title("Enter OpenAI API Key")
                .resizable(false)
                .decorations(true)
                .always_on_top(true)
                .build()
                .expect("Failed to create API Key entry window");
            }
        }
        
        
        
        
        
        
        pub fn run() {
            
            //let a_embedding_generator = EmbeddingGenerator::new(Client::new());
            // let b_embedding_generator = EmbeddingGenerator::new_with_api_key("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");
            //let path = PathBuf::from("./resources/ghostwriter-selectric/vector_store/");
            
            // log::debug!("DocumentStore initialized");            
            
            tauri::Builder::default()
            .plugin(tauri_plugin_clipboard_manager::init())
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_opener::init())
            .plugin(tauri_plugin_fs::init())
            .plugin(
                tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),       // Log to stdout
                    Target::new(TargetKind::LogDir {       // Log to app's log directory
                    file_name: None                     // Use default app name
                }),
                Target::new(TargetKind::Webview),      // Log to webview console
                ])
                .level(log::LevelFilter::Debug)
                .build()
            )
            .menu(|window| menu::build_menu(&window.app_handle()))
            .on_menu_event(|app, event| menu::handle_menu_event(app, event))
            .setup(move |app| {
                let app_handle = app.handle();
                log::info!("Ghostwriter starting up");
                log::info!("Initializing components...");
                
                // Get and store resource paths globally
                if let Ok(resource_path) = app.path().resolve("./resources/libpdfium.dylib", BaseDirectory::Resource) {
                    log::debug!("Resource path: {:?}", resource_path);
                    *PDF_LIB_PATH.lock().unwrap() = Some(resource_path);
                }
                
                if let Ok(resource_dir) = app.path().resource_dir() {
                    log::debug!("Resource directory: {}", resource_dir.display());
                    *RESOURCE_DIR_PATH.lock().unwrap() = Some(resource_dir);
                    
                    // List all files in the resource directory
                    // ...rest of your existing code...
                }
                
                log::debug!("BaseDirectory::Resource is {:?}", BaseDirectory::Resource);
                //let resource_path = app.path().resolve("./resources/libpdfium.dylib", BaseDirectory::Resource)?;
                log::debug!("Resource path: {:?}", get_resource_dir_path());
                log::debug!("libpdfium.dylib path: {:?}", get_pdf_lib_path());
                let resource_dir = app.path().resource_dir()?;
                log::debug!("Resource directory: {}", resource_dir.display());
                
                // List all files in the resource directory
                let entries = std::fs::read_dir(&resource_dir)?;
                for entry in entries {
                    if let Ok(entry) = entry {
                        log::debug!("Found resource: {}", entry.path().display());
                        
                        // // If you want to recursively list directories
                        // if entry.path().is_dir() {
                        //     // You could implement a recursive function here
                        //     dog::debug!("  (Directory)");
                        // }
                    }
                }
                
                let api_key = match KeychainHandler::retrieve_api_key() {
                    
                    Ok(Some(key)) => {
                        let new_logger: NewLogger = NewLogger::new(app_handle.clone());
                        new_logger.simple_log_message("Successfully retrieved API key from keyring".to_string(), "startup".to_string(), "info".to_string());
                        log::info!("Successfully retrieved API key from keyring");
                        Some(key)
                    },
                    Ok(None) => {
                        log::warn!("No API key found in keyring");
                        let new_logger: NewLogger = NewLogger::new(app_handle.clone());
                        new_logger.simple_log_message("No API key found in keyring".to_string(), "startup".to_string(), "warn".to_string());
                        None
                    },
                    Err(e) => {
                        log::error!("Failed to retrieve OpenAI API key from keyring: {}", e);
                        let new_logger: NewLogger = NewLogger::new(app_handle.clone());
                        new_logger.simple_log_message("Failed to retrieve OpenAI API key from keyring".to_string(), "startup".to_string(), "error".to_string());
                        None
                    } 
                };
                
                let b_embedding_generator = if let Some(key) = api_key {
                    // We have a key, create generator with it
                    log::debug!("Initializing EmbeddingGenerator with API key");
                    EmbeddingGenerator::new_with_api_key(&key)
                } else {
                    // No key found, create default generator
                    log::warn!("No API key available, initializing EmbeddingGenerator without key");
                    EmbeddingGenerator::new()
                };
                let path = app.path().app_data_dir().expect("This should never be None");
                let path = path.join("./canon/");
                
                //load_openai_api_key_from_keyring(app_handle.clone(), );
                
                let embedding_generator_clone: EmbeddingGenerator = b_embedding_generator.clone();
                
                let doc_store = match DocumentStore::new(path.clone()) {
                    Ok(store) => store,
                    Err(e) => {
                        let error_msg = format!("Failed to initialize document store at path: {:?}. Error: {}", path, e);
                        log::error!("{}", error_msg);
                        let new_logger = NewLogger::new(app_handle.clone());
                        new_logger.simple_log_message(
                            error_msg.clone(),
                            "startup".to_string(),
                            "error".to_string()
                        );
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            error_msg
                        )));
                    }
                };
                
                let store_name = doc_store.get_database_name().to_string();
                let store_path = doc_store.get_database_path().to_string();
                let app_state = AppState::new(
                    doc_store,
                    embedding_generator_clone,
                    "/tmp/gh-log.json",
                    app_handle.clone()
                ).expect("Failed to create AppState");
                
                
                app.manage(app_state);
                //let foo = app.state::<AppState>();
                
                //log::debug!("AppState managed? {:?}", foo);
                
                
                let new_logger = NewLogger::new(app_handle.clone());
                app.manage(new_logger);
                // app_state.update_logger_path(app_handle.path().app_local_data_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy().to_string()).expect("Failed to update logger path");
                //println!("{}", app_handle.path().app_local_data_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
                // app.manage(doc_store);
                // app.manage(new_logger.clone());
                // Load .env file
                //dotenv::dotenv().ok();
                
                let state = app.state::<AppState>();
                load_preferences(app_handle.clone(), state.clone());
                Ok(()
            )
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            completion_from_context,
            search_similarity,
            ingestion_from_file_dialog,
            test_log_emissions,
            simple_log_message,
            rich_log_message,
            delete_canon_entry,
            save_openai_api_key_to_keyring,
            load_openai_api_key_from_keyring,
            list_canon_docs,
            load_preferences,
            update_preferences,
            reset_preferences,
            prefs_file_path,
            get_logger_path,
            set_logger_app_data_path,
            get_log_contents,
            get_canon_info,
            save_text_content,
            save_json_content,
            ingest_from_url,
            turn_on_vibrancy,
            get_model_names,
            toggle_rag_pause,
            shot_clock_complete,
            open_canon_control_panel,
            close_canon_control_panel,
            toggle_canon_control_panel,
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
            
            
            
        }