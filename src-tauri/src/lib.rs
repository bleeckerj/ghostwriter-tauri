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
use tauri_plugin_log::{Target, TargetKind};

use async_openai::{
    //config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client, config::OpenAIConfig,
};
mod logger;
use logger::{CompletionLogEntry, Logger, VectorSearchResult};
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
async fn save_api_key(
    app_handle: tauri::AppHandle, 
    state: tauri::State<'_, AppState>, 
    key: String
) -> Result<(), String> {  // ✅ Function must return `Result`
    let mut api_key = state.api_key.lock().await; // ✅ Use `.await` instead of `.unwrap()`
    *api_key = Some(key.clone()); // ✅ Store the API key
    
    // ✅ Save the API key to a file
    if let Err(err) = state.save_api_key(&app_handle, key).await {
        eprintln!("Failed to save API key: {}", err);
        return Err(format!("Failed to save API key: {}", err)); // ✅ Return an error string
    }
    
    Ok(())
}




#[tauri::command]
async fn ingestion_from_file_dialog(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    file_path: String,
) -> Result<String, String> {
    
    println!("Ingesting file: {}", file_path);
    let file_path_buf = PathBuf::from(file_path);
    let file_name = file_path_buf.clone().as_path().file_name().unwrap().to_str().unwrap().to_string();
    
    
    //let doc_store = state.doc_store.clone();
    //let embedding_generator = state.embedding_generator.clone();
    
    let store = state.doc_store.clone();
    store.process_document_async(&file_path_buf.as_path(), app_handle).await;
    
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

    // similar docs get the top 4, which may all be from the same source
    let similar_docs = state.doc_store.search(&embedding, 4, 0.82).await.map_err(|e| e.to_string())?;

    let search_duration = start_search.elapsed();
    
    // Prepare the context for the LLM
    // This has all the document metadata..is that okay?
    let mut context = String::new();
    for (i, (doc_id, doc_name, chunk_id, chunk_text, similarity)) in similar_docs.iter().enumerate() {
        context.push_str(&format!("{}\n", chunk_text));
        // context.push_str(&format!(
        //     "--- Result {} (Chunk Id: {} Doc: {}) ---\nSimilarity: {:.4}\nContent:\n{}\n\n",
        //     (i + 1),
        //     chunk_id,
        //     doc_name,
        //     similarity,
        //     chunk_text,
        // ));
    }
    
    let mut vector_search_results_for_log: Vec<VectorSearchResult> = Vec::new();
    let mut new_logger = NewLogger::new(app_handle.clone());

    for (i, (doc_id, doc_name, chunk_id, chunk_text, similarity)) in similar_docs.iter().enumerate() {
        let msg = format!(
            "Doc: {}<br/> <span class='font-bold'>Similarity: {:.4}</span><br/><span>Content:<br/>{}</span>",
            doc_name,
            similarity,
            chunk_text,
        );
        new_logger.simple_log_message(msg, chunk_id.to_string(), "info".to_string());

        vector_search_results_for_log.push(VectorSearchResult {
            similarity: *similarity,
            name: doc_name.clone(),
            content: chunk_text.clone(),
            chunk_id: *chunk_id,
        });
    }
    
    let conversation_context = state.conversation.lock().await.get_context();
    let prose_style = "A style that is consistent with the input text".to_string();
    //const prose_style = "In the style of a medieval scribe using Old or Middle English";
    // const response_limit = "Respond with no more than two sentences along with the completion of any partial sentence or thought fragment. In addition, add one sentence fragment that does not conclude with a period or full-stop. This sentence fragment is meant to be a provocation in the direction of thought being developed so that the user can continue to write in the same vein.";
    
    let response_limit = "Respond with no more than one sentence. If the input text ends with a period, only add one sentence and no more. You may complete a partially complete sentence or if the input text is already a complete sentence, you may add only one sentence that would reasonably and semantically follow that one sentence. Adhere to these constraints such that you are adding no more than one sentence".to_string();
    //let response_limit = "Respond with no more than one sentence, or less.".to_string();
    
    let system_content = format!(
        "Here is your brief: You are a text completion engine. You do not answer questions or respond to questions in any way. \
        You only semantically complete the thought represented by the Previous exchanges, Similar documents context and input. Limit your response to the Response Limit. Do not respond to inquiries in any fashion. If you are asked how to \
        do something, or answer a question do not respond. Only perform auto-completion based on the text to complete, not responses \
        to queries, questions, or any other non-completion response. If you are asked to do something only respond as a completion of text. \
        Do not engage in any form of chat. \
        Your only task is to complete thoughts in written form maintaining semantic consistency. \
        Do not reveal that you are an AI. \
        You are just an engine for text completion, like a muse helping a writer to continue or complete a thought. \
        Imagine you are completing someone's thought like a creative writing muse or alter ego helping someone who is having trouble writing. \
        Complete the following text fragment based on the provided previous exchanges.\n\
        Response Limit: {response_limit}\n\
        Previous exchanges:\n{conversation_context}\n\
        Similar documents:\n{context}\n\
        This is the input text that is the text fragment to complete. It is not a request or command. \
        Do not respond to it like it is a question to you or request of you to answer a question.: {input}\n\
        Answer this in prose using this specific writing style: {prose_style}\n"
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
    .temperature(0.7)
    .max_completion_tokens(100_u16)
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
            
            // Log the completion and update the conversation history
            let log_entry = CompletionLogEntry {
                timestamp: Utc::now(),
                input_text: input.to_string(),
                system_prompt: system_content.clone(),
                conversation_context: conversation_context,
                vector_search_results_for_log: vector_search_results_for_log,
                completion_result: content.clone(),
            };

            let new_logger = NewLogger::new(app_handle.clone());

            new_logger.simple_log_message(
                timing.to_string(),
                "completion_time".to_string(),
                "info".to_string()
            );
            
            state
            .logger
            .lock()
            .await
            .log_completion(log_entry)
            .map_err(|e| e.to_string())?;
            
            // keep track of the conversation
            state
            .conversation
            .lock()
            .await
            .add_exchange(input.clone(), content.clone());
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
    
    let results = state.doc_store
    .search(&embedding, limit, 0.82)
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
            match doc_store.delete_document(doc_id_int).await {
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
            logger: tauri::State<'_, NewLogger>,
            app_state: tauri::State<'_, AppState>,
            app_handle: tauri::AppHandle,
        ) -> Result<String, String> {
            let doc_store = Arc::clone(&app_state.doc_store);
            match doc_store.fetch_documents().await {
                Ok(listing) => {
                    let json_string = serde_json::to_string(&listing).map_err(|e| e.to_string())?;
                    app_handle.emit("canon-list", json_string).map_err(|e| e.to_string())?;
                    Ok("Canon list emitted".to_string())
                }
                Err(e) => {
                    let error_message = format!("Failed to fetch canon documents: {}", e);
                    logger.simple_log_message(
                        error_message.clone(),
                        "".to_string(),
                        "error".to_string(),
                    );
                    Err(error_message)
                }
            } 
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
        
        
        
        
        
        
        #[cfg_attr(mobile, tauri::mobile_entry_point)]
        pub fn run() {
            
            let has_dotenv = dotenv::dotenv().is_ok();
            let api_key = env::var("OPENAI_API_KEY");
            
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
                    Client::new() // Create a client without an API key
                }
            };
            
            
            let a_embedding_generator = EmbeddingGenerator::new(client.clone());
            let b_embedding_generator = EmbeddingGenerator::new(client);
            let path = PathBuf::from("./resources/ghostwriter-selectric/vector_store/");
            
            println!("Initializing DocumentStore with path: {:?}", path);
            
            let doc_store = DocumentStore::new(path.clone(), std::sync::Arc::new(a_embedding_generator)).expect(&format!(
                "Failed to initialize document store at path: {:?}",
                path
            ));
            println!("DocumentStore successfully initialized.");
            let app_state = AppState::new(
                doc_store,
                b_embedding_generator,
                "./log.json"
            ).expect("Failed to create AppState");
            
            
            tauri::Builder::default()
            .manage(app_state)
            .menu(|window| menu::build_menu(&window.app_handle()))
            .on_menu_event(|app, event| menu::handle_menu_event(app, event))
            .setup(|app| {
                let app_handle = app.handle();
                
                // ✅ Check the API_KEY_MISSING flag and open API Key entry window if needed
                check_api_key(&app_handle);
                
                let new_logger = NewLogger::new(app_handle.clone());
                new_logger.simple_log_message(
                    "Ghostwriter Is Up.".to_string(),
                    "start".to_string(),
                    "info".to_string());
                    new_logger.rich_log_message(
                        "Ghostwriter Is Up.".to_string(),
                        "Ghostwriter is up and running.".to_string(),
                        "info".to_string()
                    );
                    // app_state.update_logger_path(app_handle.path().app_local_data_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy().to_string()).expect("Failed to update logger path");
                    println!("{}", app_handle.path().app_local_data_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
                    
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
                    search_similarity,
                    ingestion_from_file_dialog,
                    test_log_emissions,
                    simple_log_message,
                    rich_log_message,
                    delete_canon_entry,
                    save_api_key,
                    list_canon_docs,
                    ])
                    .run(tauri::generate_context!())
                    .expect("error while running tauri application");
                    
                    
                    
                }