#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use epub::doc;
use pdf_extract::Path;
use std::io::Stdout;
use std::path::PathBuf;
use std::sync::Mutex;
use std::fmt;
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

mod preferences;
use preferences::Preferences;
mod keychain_handler;
use keychain_handler::KeychainHandler;


use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use embeddings::EmbeddingGenerator;
use document_store::DocumentStore;

pub mod ingest;
pub mod document_store;
pub mod menu;
pub mod embeddings;

mod conversations; // Add this line
use conversations::Conversation;

mod app_state; // Add this line
use app_state::AppState;

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
    static ref API_KEY_MISSING: Mutex<bool> = Mutex::new(false);
}

// At the top of your file with other constants
const MENU_FILE_NEW: &str = "file-new";
const MENU_FILE_QUIT: &str = "file-quit";

#[derive(Serialize)]
struct CompletionTiming {
    embedding_generation_ms: u128,
    similarity_search_ms: u128,
    openai_request_ms: u128,
    total_ms: u128,
}

impl fmt::Display for CompletionTiming {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Completion Timing:\n\
            Embedding Generation: {} ms\n\
            Similarity Search: {} ms\n\
            OpenAI Request: {} ms\n\
            Total: {} ms",
            self.embedding_generation_ms,
            self.similarity_search_ms,
            self.openai_request_ms,
            self.total_ms
        )
    }
}

#[tauri::command]
async fn load_openai_api_key_from_keyring(
    app_handle: tauri::AppHandle, 
    state: tauri::State<'_, AppState>) -> Result<(bool), String> {
    // Create logger instance
    let new_logger = NewLogger::new(app_handle.clone());

    // Returns true if there was a key there (even if it was invalid)
    // Returns false if there was no key there
    // Returns an error if there was a problem loading the key
    match KeychainHandler::retrieve_api_key() {
        Ok(key) => {
            match key {
                Some(k) => {
                    log::info!("API key successfully loaded from keychain");
                    new_logger.simple_log_message(
                        format!("{} API key successfully loaded from keychain", k),
                        "keychain".to_string(),
                        "debug".to_string()
                    );
                    Ok(true)
                }
                None => {
                    log::info!("No API key found in keychain");
                    new_logger.simple_log_message(
                        "No API key not found in keychain".to_string(),
                        "keychain".to_string(),
                        "debug".to_string()
                    );
                    Ok(false)
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
) -> Result<(Preferences), String> {
    
    println!("update_preferences called with: {}, {}, {}, {} {} {} {} {} {} {}",
    responselimit, mainprompt, finalpreamble, prosestyle, similaritythreshold, shufflesimilars, similaritycount, maxhistory, maxtokens, temperature);
    
    let mut preferences = state.preferences.lock().await;
    preferences.response_limit = responselimit;
    preferences.main_prompt = mainprompt;
    preferences.final_preamble = finalpreamble;
    preferences.prose_style = prosestyle;
    preferences.similarity_threshold = similaritythreshold.parse::<f32>().unwrap() / 100.0;
    preferences.shuffle_similars = shufflesimilars == true;
    preferences.similarity_count = similaritycount.parse::<usize>().unwrap_or(Preferences::SIMILARITY_COUNT_DEFAULT);
    preferences.max_history = maxhistory.parse::<usize>().unwrap_or(Preferences::MAX_HISTORY_DEFAULT);
    preferences.max_output_tokens = maxtokens.parse::<u32>().unwrap_or(Preferences::MAX_OUTPUT_TOKENS_DEFAULT);
    preferences.temperature = temperature.parse::<f32>().unwrap_or(Preferences::TEMPERATURE_DEFAULT);
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
    preferences.save().map_err(|e| e.to_string()); // ✅ Persist preferences
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
    Ok(logger.get_logger_path().to_str().unwrap().to_string())
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
    
    println!("Ingesting file: {}", file_path);
    log::debug!("Ingesting file: {}", file_path);

    let file_path_buf = PathBuf::from(file_path);
    let file_name = file_path_buf.clone().as_path().file_name().unwrap().to_str().unwrap().to_string();
    
    
    //let doc_store = state.doc_store.clone();
    //let embedding_generator = state.embedding_generator.clone();
    
    let store = state.doc_store.lock().await;
    let store_clone = Arc::new(store.clone());
    store_clone.process_document_async(&file_path_buf, app_handle).await;
    
    // tokio::spawn(async move {
    //     if let Err(err) = store.process_document_async(file_path_buf.as_path()).await {
    //         eprintln!("Error processing document: {}", err);
    //     }
    // });
    
    //doc_store.lock().unwrap().process_document_async(file_path_buf.as_path()).await;
    
    Ok("Ingested file".to_string())
}


// Modify the function return type to include timing
#[tauri::command]
async fn completion_from_context(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    input: String,
) -> Result<(String, CompletionTiming), String> {
    
    let preferences = state.preferences.lock().await;
    let max_tokens: u32 = preferences.max_output_tokens;
    let temperature = preferences.temperature;
    let shuffle_similars = preferences.shuffle_similars;
    let similarity_count = preferences.similarity_count;
    let max_history = preferences.max_history;
    let similarity_threshold = preferences.similarity_threshold;
    let mut new_logger = NewLogger::new(app_handle.clone());
    
    let start_total = Instant::now();
    
    // Time embedding generation
    let start_embedding = Instant::now();
    let embedding = state
    .embedding_generator
    .generate_embedding(&input)
    .await
    .map_err(|e| e.to_string())?;
    let embedding_duration = start_embedding.elapsed();
    
    // Time similarity search
    let start_search = Instant::now();
    let mut similar_docs: Vec<(i64, String, usize, String, f32)> = Vec::new();
    let database_name: String;
    let database_path: String;
    // similar docs get the top 4, which may all be from the same source
    // fence this off so we can release the lock on the store
    {
        let store = state.doc_store.lock().await;
        similar_docs = store.search(&embedding, similarity_count, similarity_threshold).await.map_err(|e| e.to_string())?;
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
    
    new_logger.simple_log_message(
        format!("Found {} cosine similar documents ({})", similar_docs.len(), similarity_threshold),
        "".to_string(),
        "info".to_string()
    );
    
    // Prepare the context for the LLM
    // This has all the document metadata..is that okay?
    let mut context = String::new();
    for (i, (doc_id, doc_name, chunk_id, chunk_text, similarity)) in similar_docs.iter().enumerate() {
        context.push_str(&format!("{}\n", chunk_text));
    }
    
    let mut vector_search_results_for_log: Vec<VectorSearchResult> = Vec::new();
    
    for (i, (doc_id, doc_name, chunk_id, chunk_text, similarity)) in similar_docs.iter().enumerate() {
        
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
    }
    
    let conversation_context = state.conversation.lock().await.get_context();
    
    let prose_style = preferences.prose_style.clone();
    
    // let prose_style = "A style that is consistent with the input text".to_string();
    // //const prose_style = "In the style of a medieval scribe using Old or Middle English";
    // // const response_limit = "Respond with no more than two sentences along with the completion of any partial sentence or thought fragment. In addition, add one sentence fragment that does not conclude with a period or full-stop. This sentence fragment is meant to be a provocation in the direction of thought being developed so that the user can continue to write in the same vein.";
    
    let response_limit: String = preferences.response_limit.clone();
    // let response_limit = "Respond with no more than one sentence. If the input text ends with a period, only add one sentence and no more. You may complete a partially complete sentence or if the input text is already a complete sentence, you may add only one sentence that would reasonably and semantically follow that one sentence. Adhere to these constraints such that you are adding no more than one sentence".to_string();
    //let response_limit = "Respond with no more than one sentence, or less.".to_string();
    
    let main_prompt: String = preferences.main_prompt.clone();
    // let main_prompt = "Here is your brief: You are a text completion engine. You do not answer questions or respond to questions in any way. You only semantically complete the thought represented by the Previous exchanges, Similar documents context and input. Limit your response to the Response Limit. Do not respond to inquiries in any fashion. If you are asked how to do something, or answer a question do not respond. Only perform auto-completion based on the text to complete, not responses to queries, questions, or any other non-completion response. If you are asked to do something only respond as a completion of text. Do not engage in any form of chat. Your only task is to complete thoughts in written form maintaining semantic consistency. Do not reveal that you are an AI. You are just an engine for text completion, like a muse helping a writer to continue or complete a thought. Imagine you are completing someone's thought like a creative writing muse or alter ego helping someone who is having trouble writing. Complete the following text fragment based on the provided previous exchanges.";
    // let final_preamble = "This is the input text that is the text fragment to complete. It is not a request or command. Do not respond to it like it is a question to you or request of you to answer a question.:";
    
    let final_preamble: String = preferences.final_preamble.clone();
    
    let system_content = format!("{main_prompt}
        \
        Response Limit: {response_limit}\
        \
        Previous exchanges: {conversation_context}\
        \
        Similar documents: {context}\
        \
        {final_preamble} \
        Input Text: {input}\
        \
        Answer this in prose using this specific writing style: {prose_style}"
);

// Create system and user messages for OpenAI
let system_message = ChatCompletionRequestMessage::System(
    ChatCompletionRequestSystemMessageArgs::default()
    .content(system_content.clone())
    .build()
    .map_err(|e| e.to_string())?,
);

let user_message = ChatCompletionRequestMessage::User(
    ChatCompletionRequestUserMessageArgs::default()
    .content(input.clone())
    .build()
    .map_err(|e| e.to_string())?,
);

// Create and send the OpenAI request
let request = CreateChatCompletionRequestArgs::default()
.model("chatgpt-4o-latest")
.messages(vec![system_message, user_message])
.temperature(temperature)
.max_completion_tokens(max_tokens as u32)
.n(1)
.build()
.map_err(|e| e.to_string())?;

// Time OpenAI request
let start_openai = Instant::now();
dotenv::dotenv().ok();

let openai_api_key = env::var("OPENAI_API_KEY").expect("
    OPENAI_API_KEY not found. Error.");


let has_dotenv = dotenv::dotenv().is_ok();
let api_key = env::var("OPENAI_API_KEY");
let logger_clone = state.logger.clone();
let client = match &*OPENAI_API_KEY {
    Some(key) => {
        Client::with_config(
            OpenAIConfig::new()
            .with_api_key(key.clone())
        )
    }
    None => {
        println!("OPENAI_API_KEY not found.  Running without it.");
        *API_KEY_MISSING.lock().unwrap() = true; // Set the flag
        println!("OPENAI_API_KEY not found.  Running without it.");
        //let mut logger = state.logger.lock().await;
        let mut new_logger = NewLogger::new(app_handle.clone());
        new_logger.simple_log_message(
            "OPENAI_API_KEY not found or invalid. No use running without it.".to_string(),
            "".to_string(),
            "error".to_string()
        );
        new_logger.simple_log_message(
            "Try restarting and re-entering your OpenAI API Key".to_string(),
            "".to_string(),
            "error".to_string()
        );
        Client::new() // Create a client without an API key
    }
};
let response = client
.chat()
.create(request)
.await
.map_err(|e| e.to_string())?;
let openai_duration = start_openai.elapsed();

let total_duration = start_total.elapsed();

// Process the response
if let Some(choice) = response.choices.first() {
    if let Some(content) = &choice.message.content {
        // Create timing info
        let timing = CompletionTiming {
            embedding_generation_ms: embedding_duration.as_millis(),
            similarity_search_ms: search_duration.as_millis(),
            openai_request_ms: openai_duration.as_millis(),
            total_ms: total_duration.as_millis(),
        };
        // let store: tokio::sync::MutexGuard<'_, DocumentStore> = state.doc_store.lock().await;
        // let database_name = store.get_database_name().to_string(); // Just convert &str to String
        // let database_path = store.get_database_path().to_string(); // Just convert &str to String
        
        
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
        match state.logger.lock().await.log_completion(entry) {
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
        return Ok((content.clone(), timing));
    }
}

Err("No completion returned.".to_string())
}

// Add this near your other struct definitions
#[derive(Serialize)]
struct SearchResult {
    document_name: String,
    chunk_id: usize,
    chunk_text: String,
    similarity_score: f32,
}

#[tauri::command]
async fn search_similarity(
    state: tauri::State<'_, AppState>,
    query: String,
    limit: Option<usize>,  
) -> Result<Vec<SearchResult>, String> {  // Changed return type
    let limit = limit.unwrap_or(3);
    let preferences = state.preferences.lock().await;
    
    let embedding = state
    .embedding_generator
    .generate_embedding(&query)
    .await
    .map_err(|e| format!("Embedding generation failed: {}", e))?;
    
    let doc_store = state.doc_store.clone();
    
    // let doc_store = state
    // .doc_store
    // .lock()
    // .map_err(|e| format!("Failed to acquire doc store lock: {}", e))?;
    let similarity_threshold = preferences.similarity_threshold;
    let store = state.doc_store.lock().await;
    let results = store
    .search(&embedding, limit, similarity_threshold)
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
            //logger: tauri::State<'_, NewLogger>,
            app_state: tauri::State<'_, AppState>,
            app_handle: tauri::AppHandle,
        ) -> Result<String, String> {
            let doc_store = Arc::clone(&app_state.doc_store);
            println!("Listing canon documents");
            log::debug!("Listing canon documents");
            log::debug!("doc_store is {:?}", doc_store);
            let store = doc_store.lock().await;
            log::debug!("store (locked doc_store) is {:?}", store);
            match store.fetch_documents().await {
                Ok(listing) => {
                    log::debug!("listing is {:?}", listing);
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
        
        #[derive(Serialize, Clone)]
        struct CanonInfo {
            name: String,
            path: String,
        }
        #[tauri::command]
        async fn get_canon_info(
            logger: tauri::State<'_, NewLogger>,
            app_state: tauri::State<'_, AppState>,
            app_handle: tauri::AppHandle,
            docid: String,
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
                log::debug!("{}", message);
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
                    WebviewUrl::App("api_key.html".into()) // ✅ Provide a valid URL
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
            
            
            // let has_dotenv = dotenv::dotenv().is_ok();
            // let api_key = env::var("OPENAI_API_KEY");
            
            // let client = match &*OPENAI_API_KEY {
            //     Some(key) => {
            //         Client::with_config(
            //             OpenAIConfig::new()
            //             .with_api_key(key.clone())
            //         )
            //     }
            //     None => {
            //         println!("OPENAI_API_KEY not found.  Running without it.");
            //         *API_KEY_MISSING.lock().unwrap() = true; // Set the flag
            //         Client::new() // Create a client without an API key
            //     }
            // };
            
            //let a_embedding_generator = EmbeddingGenerator::new(Client::new());
            let b_embedding_generator = EmbeddingGenerator::new_with_api_key("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");
            //let path = PathBuf::from("./resources/ghostwriter-selectric/vector_store/");
            
            
            
            
            // log::debug!("DocumentStore initialized");
            
            
            
            println!("DocumentStore successfully initialized.");
            
            
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
                log::debug!("Application starting up");
                log::info!("Initializing components...");
                // Now these log messages will work
                log::debug!("This is a debug message");
                log::info!("This is an info message");
                log::warn!("This is a warning message");
                log::error!("This is an error message");
                log::trace!("This is a trace message");
                // ✅ Check the API_KEY_MISSING flag and open API Key entry window if needed
                //check_api_key(&app_handle);
                let path = app.path().app_data_dir().expect("This should never be None");
                let path = path.join("./canon/");
                let embedding_generator_clone: EmbeddingGenerator = b_embedding_generator.clone();

                let doc_store = match DocumentStore::new(path.clone(), std::sync::Arc::new(b_embedding_generator)) {
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
                    "/tmp/gh-log.json"
                ).expect("Failed to create AppState");


                app.manage(app_state);
                let foo = app.state::<AppState>();
                
                log::debug!("AppState managed? {:?}", foo);


                let new_logger = NewLogger::new(app_handle.clone());
                app.manage(new_logger);
                // app_state.update_logger_path(app_handle.path().app_local_data_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy().to_string()).expect("Failed to update logger path");
                //println!("{}", app_handle.path().app_local_data_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
                // app.manage(doc_store);
                // app.manage(new_logger.clone());
                // Load .env file
                //dotenv::dotenv().ok();

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
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
            
            
            
        }