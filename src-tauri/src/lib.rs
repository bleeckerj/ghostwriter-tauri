#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use epub::doc;
use std::io::Stdout;
use std::path::PathBuf;
use std::sync::Mutex;
use serde::Serialize;
use serde_json::Value;
use tokio::time::{sleep, Duration};
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



#[tauri::command]
async fn completion_from_context(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    input: String,
) -> Result<String, String> {
    
    // Generate embedding for the input
    let embedding = state
        .embedding_generator
        .generate_embedding(&input)
        .await
        .map_err(|e| e.to_string())?;

    // Search for similar documents
    let similar_docs = state
        .doc_store
        .lock()
        .unwrap()
        .search(&embedding, 3)
        .map_err(|e| e.to_string())?;

    // Prepare the context for the LLM
    // This has all the document metadata..is that okay?
    let mut context = String::new();
    for (i, (doc, similarity)) in similar_docs.iter().enumerate() {
        context.push_str(&format!(
            "--- Document {} (Path: {}) ---\nSimilarity: {:.4}\nContent:\n{}\n\n",
            i + 1,
            doc.path,
            similarity,
            doc.content
        ));
    }

    let mut vector_search_results: Vec<VectorSearchResult> = Vec::new();

    for (doc, similarity) in similar_docs.iter() {
        vector_search_results.push(VectorSearchResult {
            path: doc.path.clone(),
            similarity: *similarity,
            content: doc.content.clone(),
        });
    }

    let conversation_context = state.conversation.lock().unwrap().get_context();
    let prose_style = "A style that is consistent with the input text".to_string();
    //const prose_style = "In the style of a medieval scribe using Old or Middle English";
    // const response_limit = "Respond with no more than two sentences along with the completion of any partial sentence or thought fragment. In addition, add one sentence fragment that does not conclude with a period or full-stop. This sentence fragment is meant to be a provocation in the direction of thought being developed so that the user can continue to write in the same vein.";

    let response_limit = "Respond with no more than one sentence or the completion of a thought fragment. Write until you complete one thought with a full-stop or period. You may complete a partially complete sentence or if the input text is already a complete sentence, you may add a sentence that follows it.".to_string();
    
    let system_content = format!(
        "Here is your brief: You are a text completion engine. You do not answer questions or respond to questions in any way. \
        You only complete sentences, fragments, paragraphs. Do not respond to inquiries in any fashion. If you are asked how to \
        do something, or answer a question do not respond. Only perform auto-completion based on the text to complete, not responses \
        to queries, questions, or any other non-completion response. If you are asked to do something only respond as a completion of text. \
        Do not engage in any form of chat. \
        Your only task is to complete thoughts in prose. \
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
        .max_tokens(100_u16)
        .build()
        .map_err(|e| e.to_string())?;

    let response = Client::new()
        .chat()
        .create(request)
        .await
        .map_err(|e| e.to_string())?;

    // Process the response
    if let Some(choice) = response.choices.first() {
        if let Some(content) = &choice.message.content {
            // Log the completion and update the conversation history
            let log_entry = CompletionLogEntry {
                timestamp: Utc::now(),
                input_text: input.to_string(),
                system_prompt: system_content.clone(),
                conversation_context: conversation_context,
                vector_search_results: vector_search_results,
                completion_result: content.clone(),
            };

            state
                .logger
                .lock()
                .unwrap()
                .log_completion(log_entry)
                .map_err(|e| e.to_string())?;

            // keep track of the conversation
            state
                .conversation
                .lock()
                .unwrap()
                .add_exchange(input.clone(), content.clone());
            //println!("Completion: {}", content);
            return Ok(content.clone());
        }
    }

    Err("No completion returned.".to_string())
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

    let progress_indicator = ProgressIndicator {
        progress_id: "embedder".to_string(),
        current_step: "0".to_string(),
        total_steps: "4".to_string(),
        current_file: "the-myth".to_string(),
        meta: "Ingesting/Embedding".to_string(),
    };

    load_progress_indicator(&app_handle, progress_indicator);

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
            timestamp: chrono::Local::now().to_rfc3339(),
            id: Some(id.clone()),
        };
        match self.app_handle.emit("simple-log-message", simple_log_data) {
            Ok(_) => println!("Simple log emitted successfully"),
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
        "./log.txt"
    ).expect("Failed to create AppState");


    tauri::Builder::default()
    .manage(app_state)
    .setup(|app| {
        let app_handle = app.handle();
        let new_logger = NewLogger::new(app_handle.clone());
        new_logger.simple_log_message(
            "Ghostwriter Up.".to_string(),
            "start".to_string(),
            "info".to_string());
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