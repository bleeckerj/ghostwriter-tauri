use async_openai::types::AudioInput;
// src/document_store.rs
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path;
use std::path::PathBuf;
use chrono::Local; 
use serde_json;
use crate::ingest::{DocumentIngestor, IngestedDocument};
use std::path::Path;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fmt::Debug;
use crate::ai::traits::{EmbeddingProvider, PreferredEmbeddingModel, ChatCompletionProvider};
use crate::ingest::{
    pdf_ingestor::PdfIngestor,
    mdx_ingestor::MdxIngestor,
    markdown_ingestor::MarkdownIngestor,
    epub_ingestor::EpubIngestor,
    text_ingestor::TextIngestor,
    url_ingestor::UrlDocumentIngestor,
    audio_ingestor::AudioIngestor,
};
use crate::ai::{self, AIProviderError};
use crate::ai::providers::{self, ProviderType, Provider};
use crate::ai::models::{ChatCompletionRequest, ChatMessage, MessageRole, EmbeddingRequest};

use tauri::Manager; // Add this import
use tauri::Emitter;
use serde_json::json;
use tokio::runtime::Handle;
use log::{SetLoggerError, LevelFilter, info};
use crate::ingest::Resource;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Document {
    pub id: usize,
    //pub content: String,
    pub name: String,
    pub file_path: String,
    pub created_at: String,
    pub embedding_model_name: String,
    pub notes: String,
    // pub embedding: Vec<f32>
}

pub struct SimilarDocument {
    pub id: usize,
    pub name: String,
    pub content: String,
    pub similarity: f32
}

#[derive(Debug, Serialize)]
pub struct DocumentListing {
    pub documents: Vec<DocumentInfo>,
    pub canon_file: String,
    pub canon_name: String,
}

#[derive(Debug, Serialize)]
pub struct DocumentInfo {
    pub id: i64,
    pub name: String,
    pub file_path: String,
    pub created_at: String,
    pub paused: bool,
    pub embedding_model_name: String,
    pub notes: String,
    pub authors: Vec<String>,
    
}
#[derive(Debug, Clone)]
pub struct DocumentStore {
    conn: Arc<Mutex<Connection>>, // Change to tokio Mutex
    ingestors: Vec<Arc<Box<dyn DocumentIngestor>>>,
    next_id: usize,
    //embedding_generator: Arc<EmbeddingGenerator>,
    canon_name: String,
    canon_path: String,
}


impl DocumentStore {
    pub const DEFAULT_CHUNK_SIZE: usize = 1024;
    pub const DEFAULT_CHUNK_OVERLAP: usize = 200;
    
    pub fn new(
        store_path: PathBuf,
        //embedding_generator: Arc<EmbeddingGenerator>
    ) -> Result<Self, Box<dyn std::error::Error>> {
        
        let (conn, canon_path, canon_name, next_id) = 
        Self::initialize_database(&store_path)?;
        
        
        
        let mut doc_store = DocumentStore { 
            conn: Arc::new(Mutex::new(conn)),
            ingestors: Vec::new(),
            next_id,
            //embedding_generator,
            canon_path,
            canon_name,
        };        
        
        doc_store.register_ingestor(Box::new(MdxIngestor));
        
        doc_store.register_ingestor(Box::new(PdfIngestor));
        doc_store.register_ingestor(Box::new(MarkdownIngestor));
        doc_store.register_ingestor(Box::new(EpubIngestor));
        doc_store.register_ingestor(Box::new(TextIngestor));
        doc_store.register_ingestor(Box::new(UrlDocumentIngestor));
        //doc_store.register_ingestor(Box::new(AudioIngestor));
        
        
        Ok(doc_store)
    }
    
    pub async fn set_database_path(
        &mut self,
        store_path: PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (conn, canon_path, canon_name, next_id) = Self::initialize_database(&store_path)?;
        
        self.conn = Arc::new(Mutex::new(conn));
        self.next_id = next_id;
        self.canon_path = canon_path;
        self.canon_name = canon_name;
        
        Ok(())
    }
    
    fn resolve_database_path(store_path: &PathBuf) -> PathBuf {
        if store_path.is_file() {
            // If it's a file, use it directly
            store_path.to_path_buf()
        } else {
            // If it's a directory (or doesn't exist), append "ghostwriter.canon"
            store_path.join("ghostwriter.canon")
        }
    }
    
    fn initialize_database(
        store_path: &PathBuf,
    ) -> Result<(Connection, String, String, usize), Box<dyn std::error::Error>> {
        
        if store_path.is_dir() || !store_path.exists() {
            std::fs::create_dir_all(store_path)?;
        }
        
        
        let db_path = Self::resolve_database_path(store_path);
        
        log::debug!("5. db_path: {:?}", db_path);
        
        
        
        let conn = Connection::open(&db_path).map_err(|e| {
            let error_msg = format!(
                "Failed to open SQLite database at {:?}: {}. Check permissions and disk space.", 
                db_path, 
                e
            );
            log::error!("{}", error_msg);
            
            // Log specific error conditions
            match e {
                rusqlite::Error::SqliteFailure(error, Some(msg)) => {
                    log::error!("SQLite error code: {:?}, message: {}", error.code, msg);
                }
                rusqlite::Error::SqliteFailure(error, None) => {
                    log::error!("SQLite error code: {:?}", error.code);
                }
                _ => log::error!("Other SQLite error: {}", e),
            }
            
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                error_msg
            )) as Box<dyn std::error::Error>
        })?;
        let canon_path = db_path.to_string_lossy().to_string();
        let canon_name = Path::new(&canon_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "UnknownDB".to_string());
        log::debug!("7. conn: {:?}", conn);
        
        // Create tables if they don't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS documents 
            (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            title TEXT,
            authors JSON,
            created_at TEXT NOT NULL,
            file_path TEXT NOT NULL,
            paused BOOLEAN DEFAULT 0,
            embedding_model_name TEXT DEFAULT 'unknown',
            notes TEXT DEFFAULT '',
            UNIQUE(file_path, embedding_model_name)
            )",
            [],
        )?;
        
        // Replace the existing CREATE TABLE statement for embeddings
        conn.execute(
            "CREATE TABLE IF NOT EXISTS embeddings 
            (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            doc_id INTEGER NOT NULL,
            chunk TEXT NOT NULL, 
            embedding JSON NOT NULL,
            embedding_model_name TEXT DEFAULT 'unknown',
            FOREIGN KEY(doc_id) REFERENCES documents(id)
            )",
            [],
        )?;
        
        // Add the new canon table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS canon 
            (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            owner TEXT NOT NULL,
            created_at TEXT NOT NULL,
            modified_at TEXT NOT NULL,
            notes TEXT NOT NULL
            )",
            [],
        )?;
        
        // Get the highest ID for our next_id counter
        let next_id: usize = conn
        .query_row(
            "SELECT COALESCE(MAX(id) + 1, 0) FROM documents",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
        
        Ok((conn, canon_path, canon_name, next_id))
    }
    
    pub async fn add_document(
        &mut self,
        mut document: Document,
    ) -> Result<(), Box<dyn std::error::Error>> {
        document.id = self.next_id;
        let current_time = Local::now().to_rfc3339();
        
        // Hold the lock for the duration of the transaction
        let mut conn_guard = self.conn.lock().await;
        let tx = conn_guard.transaction()?;
        
        // Insert document
        tx.execute(
            "INSERT INTO documents (id, name, created_at, file_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![document.id, document.name, current_time, document.file_path, document.embedding_model_name, document.notes],
        )?;
        
        tx.commit()?;
        
        self.next_id += 1;
        Ok(())
    }
    
    pub async fn search(
        &self,
        query_embedding_result: &Result<Vec<ai::models::Embedding>, AIProviderError>,
        provider: &Provider,
        similar_docs_count: usize,
        similarity_threshold: f32,
    ) -> Result<Vec<(i64, String, usize, String, f32)>, Box<dyn std::error::Error>> {
        // Handle the Result type for query_embedding
        let query_embedding = match query_embedding_result {
            Ok(embeddings) => {
                // Assuming you want to use the first embedding in the vector
                if let Some(first_embedding) = embeddings.first() {
                    &first_embedding.vector
                } else {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "No embeddings found in the result",
                    )));
                }
            }
            Err(e) => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Error obtaining embeddings: {}", e),
                )));
            }
        };
        let embedding_model_name = provider.get_preferred_embedding_model();
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT d.id, d.name, d.file_path, d.created_at, e.id, e.chunk, e.embedding 
            FROM documents d 
            JOIN (
                SELECT id, doc_id, chunk, embedding, embedding_model_name 
                FROM embeddings
                GROUP BY id, doc_id, chunk, embedding, embedding_model_name
            ) e ON d.id = e.doc_id
            WHERE (d.paused = 0 OR d.paused IS NULL) 
            AND e.embedding_model_name = ?1"
        )?;  // Use ? directly for rusqlite::Error
        
        let mut similarities = Vec::new();
        
        let rows = stmt.query_map(params![embedding_model_name], |row| {
            let doc_id: i64 = row.get(0)?;  // Extract the document id
            let name: String = row.get(1)?;  // Use ? directly for rusqlite errors
            let chunk_id: usize = row.get(4)?;
            let chunk: String = row.get(5)?;
            let embedding_json: String = row.get(6)?;
            
            let chunk_embedding: Vec<f32> = serde_json::from_str(&embedding_json)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                6,
                rusqlite::types::Type::Text,
                Box::new(e)
            ))?;
            
            let similarity = cosine_similarity(query_embedding, &chunk_embedding);
            
            Ok((doc_id, name, chunk_id, chunk, similarity))
        })?;  // Use ? directly for rusqlite::Error
        
        
        for row in rows {
            similarities.push(row?);
        }
        
        // Filter by min_score and sort by similarity score in descending order
        similarities.retain(|&(doc_id, ref name, _, _, similarity)| {
            similarity >= similarity_threshold
        });
        
        similarities.sort_by(|a, b| {
            b.4.partial_cmp(&a.4)  // Changed from .3 to .4 to access similarity
            .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Collect top results with unique doc_id and unique chunk_id
        let mut unique_results = Vec::new();
        let mut seen_doc_ids = std::collections::HashSet::new();
        let mut seen_chunk_ids = std::collections::HashSet::new();
        
        for result in &similarities {
            if seen_doc_ids.len() >= similar_docs_count {
                break;
            }
            if seen_doc_ids.insert(result.0) && seen_chunk_ids.insert(result.2) {
                unique_results.push(result.clone());
            }
        }
        
        // If we have fewer than similar_docs_count unique results, add more entries from the remaining items
        if unique_results.len() < similar_docs_count {
            for result in &similarities {
                if unique_results.len() >= similar_docs_count {
                    break;
                }
                if seen_doc_ids.insert(result.0) && seen_chunk_ids.insert(result.2) {
                    unique_results.push(result.clone());
                }
            }
        }
        
        Ok(unique_results)
    }
    
    pub async fn fetch_documents(&self) -> Result<DocumentListing, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT id, name, file_path, created_at, paused, embedding_model_name, notes, authors FROM documents")?;
        
        let rows = stmt.query_map([], |row| {
            // Parse authors from JSON string to Vec<String>
            let authors_json: String = row.get(7).unwrap_or_default();
            let authors: Vec<String> = match serde_json::from_str(&authors_json) {
                Ok(parsed) => parsed,
                Err(_) => Vec::new() // Default to empty vector if JSON parsing fails
            };
            
            Ok(DocumentInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                file_path: row.get(2)?,
                created_at: row.get(3)?,
                paused: row.get(4).unwrap_or(false),
                embedding_model_name: row.get(5).unwrap_or("unknown".to_string()),
                notes: row.get(6).unwrap_or("".to_string()),
                authors, // A Vec<String> parsed from JSON
            })
        })?;
        
        let documents: Vec<DocumentInfo> = rows.collect::<Result<_, _>>()?;
        
        // Get the database file path from the connection
        let db_path = conn.path().unwrap_or_default();
        let canon_file = std::path::Path::new(db_path)
        .file_name()  // Get just the filename
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
        let canon_name = std::path::Path::new(&canon_file)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
        
        Ok(DocumentListing {
            documents,
            canon_file,
            canon_name,
        })
    }
    
    pub fn register_ingestor(&mut self, ingestor: Box<dyn DocumentIngestor>) {
        self.ingestors.push(Arc::new(ingestor));
    }
    
    pub async fn save_document_to_file(
        &self,
        resource: &Resource,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Find suitable ingestor
        let ingestor = match self.ingestors.iter().find(|i| i.can_handle(resource)) {
            Some(ingestor) => ingestor,
            None => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No suitable ingestor found",
                )));
            }
        };
        
        // Ingest the document
        let ingested_document = ingestor.ingest(resource).await?;
        
        // Check if the ingestor is UrlDocumentIngestor and call save_to_file
        if let Some(url_ingestor) = ingestor.as_any().downcast_ref::<UrlDocumentIngestor>() {
            url_ingestor.save_to_file(&ingested_document, file_path)?;
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Ingestor does not support saving to file",
            )));
        }
        
        Ok(())
    }
    
    pub async fn ingest_url_async(
        self: Arc<Self>, 
        url: &str,
        provider: &Provider,
        app_handle: tauri::AppHandle, // Add app_handle parameter
    ) -> Result<(IngestedDocument), Box<dyn std::error::Error>> {
        
        let store = self.clone(); // Clone Arc to get a reference

        let embedding_model_name = provider.get_preferred_embedding_model();
        
        // Create a Resource::Url from the URL string
        let url_resource = Resource::Url(url.to_string());
        
        // Find suitable ingestor
        let ingestor = match store.ingestors.iter().find(|i| i.can_handle(&url_resource)) {
            Some(ingestor) => ingestor,
            None => {
                let error_message = format!("No suitable ingestor found for URL: {}", url);
                app_handle.emit("simple-log-message", json!({
                    "message": error_message,
                    "timestamp": chrono::Local::now().to_rfc3339(),
                    "level": "warn"
                }))?;
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, error_message)));
            }
        };        
        let ingested = ingestor.ingest(&url_resource).await?;
        //println!("Ingested document: {:?}", ingested);
        log::debug!("Ingested document: {:?}", ingested);
        app_handle.emit("simple-log-message", json!({
            "message": format!("Ingested document: {}", ingested.title),
            "timestamp": chrono::Local::now().to_rfc3339(),
            "level": "debug"
        }))?;
        

        let document = Document {
            id: 0,
            name: ingested.title.clone(),
            created_at: chrono::Local::now().to_rfc3339(),
            file_path: ingested.metadata.source_path.clone(),
            embedding_model_name: embedding_model_name,
            notes: "".to_string(),
        };
        let doc_name = document.name.clone();

        let doc_id_result = {
            let conn = store.conn.lock().await;
            store.add_document_internal(&conn, document.clone())
        };
        let doc_id = match doc_id_result {
            Ok(id) => {
                log::debug!("Document created: {:?}", &document);
                app_handle.emit("simple-log-message", json!({
                    "message": format!("Document created: {}", doc_name),
                    "timestamp": chrono::Local::now().to_rfc3339(),
                    "level": "debug"
                }))?;
                println!("Document ID: {}", id);
                log::debug!("Document ID: {}", id);
                id
            }
            Err(e) => {
                let error_message = format!("Error adding document to database: {}", e);
                println!("Error adding document to database: {:?}", e);
                app_handle.emit("simple-log-message", json!({
                    "message": format!("Couldn't add {} to canon because {}", doc_name, error_message),
                    "timestamp": chrono::Local::now().to_rfc3339(),
                    "level": "warn"
                })).ok();
                return Err(e); // Propagate the error
            }
        };
        
        
        
        
        
        

        
        drop(doc_id_result); // Release the lock
        app_handle.emit("simple-log-message", json!({
            "message": format!("Document added to the Canon with ID: {}", doc_id),
            "timestamp": chrono::Local::now().to_rfc3339(),
            "level": "info"
        }))?;
        let file_name = url.to_string();
        let file_name_clone = file_name.clone();
        // let app_handle_clone = app_handle.clone();
        // let name = file_name.clone();
        match store
        .process_embeddings(doc_id
            , ingested.content.clone(), file_name, &provider, app_handle.clone())
            .await
            {
                Ok(_) => {
                    println!("Document processed with ID: {}", doc_id);
                    app_handle.emit("simple-log-message", json!({
                        "message": format!("Ingestion & embedding complete: {}", file_name_clone),
                        "timestamp": chrono::Local::now().to_rfc3339(),
                        "level": "info"
                    })).ok();
                }
                Err(e) => {
                    println!("Error processing embeddings: {:?}", e);
                    app_handle.emit("simple-log-message", json!({
                        "message": format!("Ingestion & embedding complete: {}", file_name_clone),
                        "timestamp": chrono::Local::now().to_rfc3339(),
                        "level": "warn"
                    }));
                    log::warn!("Error processing embeddings: {:?}", e);
                }
            }
            
            println!("Document processed with ID: {}", doc_id);
            
            
            
            Ok((ingested))
        }
        
        
        pub async fn process_document_async(
            self: Arc<Self>, 
            provider: &Provider,
            path: &Path,
            app_handle: tauri::AppHandle, // Add app_handle parameter
        ) -> Result<(), Box<dyn std::error::Error>> {
            let store = self.clone(); // Clone Arc to get a reference
            
            // Find suitable ingestor
            let resource = Resource::FilePath(path.to_path_buf());
            let ingestor = match store.ingestors.iter().find(|i| i.can_handle(&resource)) {
                Some(ingestor) => ingestor,
                None => {
                    let error_message = "No suitable ingestor found".to_string();
                    println!("{}", error_message);
                    app_handle.emit("error", json!({
                        "message": error_message,
                        "timestamp": chrono::Local::now().to_rfc3339(),
                        "level": "error"
                    }))?;
                    println!("Ingestor not found");
                    app_handle.emit("simple-log-message", json!({
                        "message": format!("No ingestor found for {}", path.to_string_lossy()),
                        "timestamp": chrono::Local::now().to_rfc3339(),
                        "level": "warn"
                    }))?;
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, error_message)));
                }
            };
            
            app_handle.emit("simple-log-message", json!({
                "message": format!("Ingestor found: {:?}", ingestor),
                "timestamp": chrono::Local::now().to_rfc3339(),
                "level": "debug"
            }))?;
            //println!("Ingestor found");
            let resource = Resource::FilePath(path.to_path_buf());
            
            //
            // let ingested = ingestor.ingest(&resource).await?;
            //
            
            // With this more detailed error handling:
            let ingested = match ingestor.ingest(&resource).await {
                Ok(result) => {
                    app_handle.emit("simple-log-message", json!({
                        "message": "Performing special handling for URL document",
                        "timestamp": chrono::Local::now().to_rfc3339(),
                        "level": "debug"
                    }))?;
                    if let Some(url_ingestor) = ingestor.as_any().downcast_ref::<UrlDocumentIngestor>() {
                        url_ingestor.save_to_file(&result, path.to_str().unwrap())?;
                    }
                    result
                },
                Err(e) => {
                    // Log the error with details
                    log::error!("Document ingestion failed for {}: {}", path.display(), e);
                    
                    // Send error to frontend for user feedback
                    app_handle.emit("simple-log-message", json!({
                        "message": format!("Failed to process '{}': {}", path.file_name().unwrap_or_default().to_string_lossy(), e),
                        "timestamp": chrono::Local::now().to_rfc3339(),
                        "level": "error"
                    }))?;
                    
                    // Add specific handling for PDF-related errors
                    if path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("pdf")) {
                        let pdf_help = "This might be due to missing PDFium library. \
                Make sure the application resources include libpdfium for your platform.";
                        
                        app_handle.emit("simple-log-message", json!({
                            "message": pdf_help,
                            "timestamp": chrono::Local::now().to_rfc3339(),
                            "level": "warn"
                        }))?;
                        
                        log::warn!("{}", pdf_help);
                    }
                    
                    // Still return the error to be handled upstream
                    return Err(Box::new(e));
                }
            };
            
            //println!("Ingested document: {:?}", ingested);
            let document = Document {
                id: 0,
                name: ingested.title,
                created_at: chrono::Local::now().to_rfc3339(),
                file_path: ingested.metadata.source_path,
                embedding_model_name: "dk".to_string(),
                notes: "".to_string(),
                //embedding: vec![],
            };
            let doc_name = document.name.clone();
            //println!("Document created: {:?}", document);
            log::debug!("Document created: {:?}", document);
            
            let doc_id_result = {
                let conn = store.conn.lock().await;
                store.add_document_internal(&conn, document)
            };
            let doc_id = match doc_id_result {
                Ok(id) => {
                    println!("Document ID: {}", id);
                    log::debug!("Document ID: {}", id);
                    id
                }
                Err(e) => {
                    let error_message = format!("Error adding document to database: {}", e);
                    println!("Error adding document to database: {:?}", e);
                    app_handle.emit("simple-log-message", json!({
                        "message": format!("Couldn't add {} to canon {}", doc_name, error_message),
                        "timestamp": chrono::Local::now().to_rfc3339(),
                        "level": "warn"
                    })).ok();
                    return Err(e); // Propagate the error
                }
            };
            drop(doc_id_result); // Release the lock
            app_handle.emit("simple-log-message", json!({
                "message": format!("Document added to the Canon with ID: {}", doc_id),
                "timestamp": chrono::Local::now().to_rfc3339(),
                "level": "info"
            }))?;
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let file_name_clone = file_name.clone();
            // let app_handle_clone = app_handle.clone();
            // let name = file_name.clone();
            /****************************************/
            /****************************************/
            /* HERE IS WHERE EMBEDDINGS ARE CREATED */
            /****************************************/
            /****************************************/
            match store
            .process_embeddings(doc_id, ingested.content, file_name, &provider, app_handle.clone())
            .await
            {
                Ok(_) => {
                    println!("Document processed with ID: {}", doc_id);
                    app_handle.emit("simple-log-message", json!({
                        "message": format!("Ingestion & embedding complete: {}", file_name_clone),
                        "timestamp": chrono::Local::now().to_rfc3339(),
                        "level": "info"
                    })).ok();
                }
                Err(e) => {
                    println!("Error processing embeddings: {:?}", e);
                    app_handle.emit("simple-log-message", json!({
                        "message": format!("Ingestion & embedding complete: {}", file_name_clone),
                        "timestamp": chrono::Local::now().to_rfc3339(),
                        "level": "warn"
                    }));
                    log::warn!("Error processing embeddings: {:?}", e);
                }
            }
            
            println!("Document processed with ID: {}", doc_id);
            
            
            
            Ok(())
        }
        
        pub async fn delete_document(&self, doc_id: i64) -> Result<(), Box<dyn std::error::Error>> {
            let mut conn = self.conn.lock().await;
            
            // Start a transaction to ensure atomicity
            let tx = conn.transaction()?;
            
            // Delete embeddings associated with the document
            tx.execute("DELETE FROM embeddings WHERE doc_id = ?1", params![doc_id])?;
            
            // Delete the document itself
            tx.execute("DELETE FROM documents WHERE id = ?1", params![doc_id])?;
            
            // Commit the transaction
            tx.commit()?;
            
            Ok(())
        }
        
        pub fn get_database_path(&self) -> &str {
            &self.canon_path  // ✅ Already stored, just return it
        }
        
        pub fn get_database_name(&self) -> &str {
            &self.canon_name  // ✅ Already stored, just return it
        }
        
        /***
        * ## USAGE
        
        let canon_id_to_update: i64 = 123; // Replace with the actual canon_id
        let new_name = "New Canon Name".to_string();
        let new_owner = "New Owner".to_string();
        let new_notes = Some("Some new notes".to_string()); // Or None if you want to clear the notes
        
        let result = document_store
        .update_canon(canon_id_to_update, new_name, new_owner, new_notes)
        .await;
        
        match result {
        Ok(_) => println!("Canon with ID {} updated successfully", canon_id_to_update),
        Err(e) => eprintln!("Error updating canon with ID {}: {}", canon_id_to_update, e),
        }
        */
        pub async fn update_canon(
            &self,
            canon_id: i64,
            name: String,
            owner: String,
            notes: Option<String>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let conn = self.conn.lock().await;
            let modified_at = Local::now().to_rfc3339();
            
            let mut stmt = conn.prepare(
                "UPDATE canon SET name = ?1, owner = ?2, notes = ?3, modified_at = ?4 WHERE id = ?5",
            )?;
            
            stmt.execute(params![name, owner, notes, modified_at, canon_id])?;
            
            Ok(())
        }
        
        
        async fn process_embeddings(
            &self, 
            doc_id: i64, 
            content: String,
            file_name: String,
            provider: &Provider,
            //embedding_generator: &EmbeddingGenerator,
            app_handle: tauri::AppHandle,
        ) -> Result<(), Box<dyn std::error::Error>> {
            //println!("app_handle: {:?}", app_handle);
            
            
            let conn = self.conn.lock().await;
            let embedding_model = provider.get_preferred_embedding_model();
            
            // Update the embedding_model_name column in the documents table
            conn.execute(
                "UPDATE documents SET embedding_model_name = ?1 WHERE id = ?2",
                params![embedding_model, doc_id],
            )?;
            // Chunk the content
            let chunks = chunk_text(&content, Self::DEFAULT_CHUNK_SIZE, Self::DEFAULT_CHUNK_OVERLAP); // adjust size/overlap as needed
            
            // Emit progress update
            app_handle.emit("progress-indicator-load", json!({
                "progress_id": format!("embedding_doc_id_{}",doc_id),
                "current_step": 0,
                "total_steps": chunks.len() + 1,
                "current_file": file_name,
                "meta": content.chars().take(50).collect::<String>(),
            }))?;
            
            // Get embeddings for each chunk
            for (count, chunk) in chunks.iter().enumerate() {
                let count = count + 1;
                app_handle.emit("progress-indicator-update", json!({
                    "progress_id": format!("embedding_doc_id_{}", doc_id),
                    "current_step": count+1,
                    "total_steps": chunks.len() + 1,
                    "current_file": file_name,
                    "meta": chunk,
                }))?;
                
                
                
                //let embedding = embedding_generator.generate_embedding(app_handle.clone(), &chunk).await?;
                let embedding_request = EmbeddingRequest {
                    model: embedding_model.to_string(),
                    input: vec![chunk.clone()],
                };
                
                let embedding_response = provider.create_embeddings(embedding_request).await?;
                let embedding = embedding_response.into_iter().next().ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Other, "No embedding data received")
                })?;
                
                // Deserialize the embedding JSON to a serde_json::Value
                let embedding_value: serde_json::Value = serde_json::to_value(&embedding)?;
                
                // Extract the vector field
                let vector_value = embedding_value.get("vector").ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Other, "No vector field in embedding data")
                })?;
                
                // Serialize the vector field back to a string
                let vector_string = serde_json::to_string(vector_value)?;
                
                
                
                // Convert embedding to JSON string
                let embedding_json = serde_json::to_string(&embedding)?;
                //println!("Embedding: {}", embedding_json);
                // Store in database
                conn.execute(
                    "INSERT INTO embeddings (doc_id, chunk, embedding, embedding_model_name) VALUES (?1, ?2, ?3, ?4)",
                    params![doc_id, chunk, vector_string, embedding_model],
                )?;
                
            }
            // Emit final progress update
            app_handle.emit("progress-update", json!({
                "progress_id": "document-processing",
                "current_step": 3,
                "total_steps": 3,
                "current_file": file_name,
                "meta": "Completed"
            }))?;
            println!("Embeddings processed");
            app_handle.emit("simple-log-message", json!({
                "message": format!("Embeddings processed for {} with model {}", file_name, embedding_model),
                "timestamp": chrono::Local::now().to_rfc3339(),
                "level": "info"
            }))?;
            
            
            
            Ok(())
        }
        
        // Private helper function for database operations
        fn add_document_internal(&self, conn: &Connection, document: Document) -> Result<i64, Box<dyn std::error::Error>> {
            match conn.execute(
                "INSERT INTO documents (name, created_at, file_path, embedding_model_name) VALUES (?1, ?2, ?3, ?4)",
                params![
                document.name,
                document.created_at,
                document.file_path,
                document.embedding_model_name,
                ],
            ) {
                Ok(_) => {
                    // Get the ID of the last inserted row
                    let id = conn.last_insert_rowid();
                    Ok(id)
                }
                Err(e) => {
                    // Handle the error
                    Err(Box::new(e))
                }
            }
        }
        // pub async fn update_document_pause_state(&self, doc_id: i64, paused: bool) -> Result<(), String> {
        
        pub async fn update_document_pause_state(&self, doc_id: i64, paused: bool) -> Result<(), Box<dyn std::error::Error>> {
            let conn = self.conn.lock().await;
            
            // Check if paused column exists in documents table
            let has_paused_column = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('documents') WHERE name='paused'",
                [],
                |row| row.get::<_, i64>(0)
            )?;
            
            // Add the column if it doesn't exist (handles migration for older databases)
            if has_paused_column == 0 {
                log::info!("Adding paused column to documents table");
                conn.execute(
                    "ALTER TABLE documents ADD COLUMN paused BOOLEAN DEFAULT 0",
                    [],
                )?;
            }
            
            // Now we can safely update the pause state
            conn.execute(
                "UPDATE documents SET paused = ?1 WHERE id = ?2",
                params![paused, doc_id],
            )?;
            
            log::info!("Updated pause state for document {}: paused = {}", doc_id, paused);
            Ok(())
        }
        
        // Get the current pause state for a document
        pub async fn is_document_paused(&self, doc_id: i64) -> Result<bool, Box<dyn std::error::Error>> {
            let conn = self.conn.lock().await;
            
            // Check if paused column exists (just to be safe)
            let has_paused_column = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('documents') WHERE name='paused'",
                [],
                |row| row.get::<_, i64>(0)
            )?;
            
            if has_paused_column == 0 {
                // Column doesn't exist, so nothing is paused
                return Ok(false);
            }
            
            // Get the paused state
            let paused: bool = conn.query_row(
                "SELECT paused FROM documents WHERE id = ?1",
                params![doc_id],
                |row| row.get(0),
            )?;
            
            Ok(paused)
        }
        
        // Add this method
        pub fn find_ingestor(&self, path: &Path) -> Option<Arc<Box<dyn DocumentIngestor>>> {
            self.ingestors
            .iter()
            .find(|i| i.can_handle(&Resource::FilePath(path.to_path_buf())))
            .cloned()
        }
        
        
        
        pub async fn test_async_process(&self) -> Result<(), Box<dyn std::error::Error>> {
            for i in 1..=10 {
                // First do anything that needs the lock
                {
                    // Quick operation with lock
                    let _conn = self.conn.lock().await;
                    println!("Starting step {}", i);
                } // Lock dropped here
                
                // Then do async work without the lock
                println!("Processing step {}", i);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Ok(())
        }

        pub async fn update_document_details(
            &self,
            doc_id: i64,
            name: String,
            notes: String,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let conn = self.conn.lock().await;
            
            // Update the document details in the database
            conn.execute(
                "UPDATE documents SET name = ?1, notes = ?2 WHERE id = ?3",
                params![name, notes, doc_id],
            )?;
            
            log::info!("Updated document details for document {}: name = {}, notes length = {}", 
                doc_id, name, notes.len());
            
            Ok(())
        }
    }
    
    // Helper function for cosine similarity
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
    
    /// Chunks text into segments with optional overlap
    /// 
    /// * `text` - The text to chunk
    /// * `chunk_size` - Maximum size of each chunk in characters
    /// * `overlap` - Number of characters to overlap between chunks
    fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut i = 0;
        
        while i < words.len() {
            let mut chunk = String::new();
            let mut j = i;
            
            // Build chunk up to chunk_size
            while j < words.len() && (chunk.len() + words[j].len() + 1) <= chunk_size {
                if !chunk.is_empty() {
                    chunk.push(' ');
                }
                chunk.push_str(words[j]);
                j += 1;
            }
            
            chunks.push(chunk);
            
            // Move forward by chunk_size - overlap words for next iteration
            let advance = if j > i {
                ((j - i) as f32 * (1.0 - (overlap as f32 / chunk_size as f32))) as usize
            } else {
                1
            };
            i += advance.max(1);
        }
        
        chunks
    }


