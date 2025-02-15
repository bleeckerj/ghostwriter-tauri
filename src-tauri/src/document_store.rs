#![allow(unused_imports)]
#![allow(unused)]
// src/document_store.rs
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path;
use std::path::PathBuf;
use chrono::Local;  // Add this to your imports at the top
use serde_json;
use crate::ingest::DocumentIngestor;
use std::path::Path;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fmt::Debug;
use crate::embeddings::EmbeddingGenerator;  // Change this line
use crate::ingest::{
    pdf_ingestor::PdfIngestor,
    mdx_ingestor::MdxIngestor
};
use tauri::Manager; // Add this import
use tauri::Emitter;
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, Debug)]

pub struct Document {
    pub id: usize,
    //pub content: String,
    pub name: String,
    pub file_path: String,
    pub created_at: String,
    pub embedding: Vec<f32>
}

pub struct SimilarDocument {
    pub id: usize,
    pub name: String,
    pub content: String,
    pub similarity: f32
}

#[derive(Serialize)]
pub struct DocumentListing {
    pub documents: Vec<DocumentInfo>,
    pub canon_file: String,
    pub canon_name: String,
}

#[derive(Serialize)]
pub struct DocumentInfo {
    pub id: i64,
    pub name: String,
    pub file_path: String,
    pub created_at: String,
}

pub struct DocumentStore {
    conn: Arc<Mutex<Connection>>, // Change to tokio Mutex
    ingestors: Vec<Arc<Box<dyn DocumentIngestor>>>,
    next_id: usize,
    embedding_generator: Arc<EmbeddingGenerator>,
}


impl DocumentStore {
    pub fn new(
        store_path: PathBuf,
        embedding_generator: Arc<EmbeddingGenerator>
    ) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(& store_path)?;
        let db_path = store_path.join("documents.db");
        
        let conn = Connection::open(&db_path)?;
        
        // Create tables if they don't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS documents 
            (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            file_path TEXT NOT NULL UNIQUE
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS embeddings 
            (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            doc_id INTEGER NOT NULL,
            chunk TEXT NOT NULL, 
            embedding JSON NOT NULL,
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
            modified_at TEXT NOT NULL
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
        
        let mut doc_store = DocumentStore { 
            conn: Arc::new(Mutex::new(conn)),
            ingestors: Vec::new(),
            next_id,
            embedding_generator,
        };
        
        doc_store.register_ingestor(Box::new(MdxIngestor));
        doc_store.register_ingestor(Box::new(PdfIngestor));
        
        Ok(doc_store)
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
            "INSERT INTO documents (id, name, created_at, file_path) VALUES (?1, ?2, ?3, ?4)",
            params![document.id, document.name, current_time, document.file_path],
        )?;
        
        tx.commit()?;
        
        self.next_id += 1;
        Ok(())
    }
    
    pub async fn search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, usize, String, f32)>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT d.id, d.name, d.file_path, d.created_at, e.id, e.chunk, e.embedding 
             FROM documents d 
             JOIN embeddings e ON d.id = e.doc_id 
             LIMIT ?"
        )?;  // Use ? directly for rusqlite::Error
        
        let mut similarities = Vec::new();
        
        let rows = stmt.query_map([limit as i64], |row| {
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
            
            Ok((name, chunk_id, chunk, similarity))
        })?;  // Use ? directly for rusqlite::Error
        
        for row in rows {
            similarities.push(row?);
        }
        
        // Sort by similarity score in descending order
        similarities.sort_by(|a, b| {
            b.3.partial_cmp(&a.3)  // Changed from .2 to .3 to access similarity
            .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take top k results
        Ok(similarities.into_iter().take(limit).collect())
    }
    
    pub async fn fetch_documents(&self) -> Result<DocumentListing, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT id, name, file_path, created_at FROM documents")?;
        
        let rows = stmt.query_map([], |row| {
            Ok(DocumentInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                file_path: row.get(2)?,
                created_at: row.get(3)?,
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
    
    
    pub async fn process_document_async(
        self: Arc<Self>, 
        path: &Path,
        app_handle: tauri::AppHandle, // Add app_handle parameter
    ) -> Result<(), Box<dyn std::error::Error>> {
        let store = self.clone(); // Clone Arc to get a reference
        println!("Processing document: {:?}", path);

        // Find suitable ingestor
        let ingestor = match store.ingestors.iter().find(|i| i.can_handle(path)) {
            Some(ingestor) => ingestor,
            None => {
                let error_message = "No suitable ingestor found".to_string();
                println!("{}", error_message);
                app_handle.emit("error", json!({
                    "message": error_message,
                    "timestamp": chrono::Local::now().to_rfc3339(),
                    "level": "error"
                }))?;
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, error_message)));
            }
        };


        println!("Ingestor found");
        let ingested = ingestor.ingest_file(path).await?;
        println!("Ingested document: {:?}", ingested.metadata.source_path);
        //println!("Ingested document: {:?}", ingested);
        let document = Document {
            id: 0,
            name: ingested.title,
            created_at: chrono::Local::now().to_rfc3339(),
            file_path: ingested.metadata.source_path,
            embedding: vec![],
        };
        println!("Document created: {:?}", document);
        let doc_id = {
            let conn = store.conn.lock().await;  // ✅ Correctly locks the database connection
            let id = store.add_document_internal(&conn, document)?;  
            id
        };
        println!("Document ID: {}", doc_id);
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        
        let app_handle_clone = app_handle.clone();
        
        match store.process_embeddings(doc_id, ingested.content, file_name, &store.embedding_generator, app_handle_clone).await {
            Ok(_) => {
                println!("Document processed with ID: {}", doc_id);
            }
            Err(e) => {
                println!("Error processing embeddings: {:?}", e);
            }
        }
        
        println!("Document processed with ID: {}", doc_id);
        
        
        
        Ok(())
    }
    
    
    
    // pub async fn process_document(
    //     &mut self, 
    //     path: &Path,
    //     embedding_generator: &EmbeddingGenerator
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     // Find suitable ingestor
    //     println!("Processing document: {:?}", path);
    //     let ingestor = self.ingestors.iter()
    //     .find(|i| i.can_handle(path))
    //     .ok_or_else(|| "No suitable ingestor found".to_string())?;
        
    //     // Process the document
    //     let ingested = ingestor.ingest_file(path).await?;
        
    //     // Create document
    //     let document = Document {
    //         id: 0,  // This will be set by the database
    //         name: ingested.title,
    //         created_at: Local::now().to_rfc3339(),
    //         file_path: ingested.metadata.source_path,
    //         embedding: vec![],
    //     };
        
    //     // Insert document and get ID
    //     let doc_id = {
    //         let conn = self.conn.lock().await;
    //         self.add_document_internal(&conn, document)?
    //     };
        
    //     // Process and store embeddings
    //     self.process_embeddings(doc_id, ingested.content, embedding_generator).await?;
        
    //     println!("Document processed with ID: {}", doc_id);
    //     Ok(())
    // }
    
    async fn process_embeddings(
        &self, 
        doc_id: i64, 
        content: String,
        file_name: String,
         embedding_generator: &EmbeddingGenerator,
        app_handle: tauri::AppHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("app_handle: {:?}", app_handle);
        // Chunk the content
        let chunks = self.embedding_generator.chunk_text(&content, 1000, 100); // adjust size/overlap as needed
        // Emit progress update
        println!("Processing {} chunks", chunks.len());
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
            let embedding = embedding_generator.generate_embedding(&chunk).await?;
            
            // Convert embedding to JSON string
            let embedding_json = serde_json::to_string(&embedding)?;
            //println!("Embedding: {}", embedding_json);
            // Store in database
            let conn = self.conn.lock().await;
            conn.execute(
                "INSERT INTO embeddings (doc_id, chunk, embedding) VALUES (?1, ?2, ?3)",
                params![doc_id, chunk, embedding_json],
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
        Ok(())
    }
    
    // Private helper function for database operations
    fn add_document_internal(&self, conn: &Connection, document: Document) -> Result<i64, Box<dyn std::error::Error>> {
        conn.execute(
            "INSERT INTO documents (name, created_at, file_path) VALUES (?1, ?2, ?3)",
            params![
            document.name,
            document.created_at,
            document.file_path,
            ],
        )?;
        
        // Get the ID of the last inserted row
        let id = conn.last_insert_rowid();
        Ok(id)
    }
    
    // Add this method
    pub fn find_ingestor(&self, path: &Path) -> Option<Arc<Box<dyn DocumentIngestor>>> {
        self.ingestors
        .iter()
        .find(|i| i.can_handle(path))
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
    
    #[cfg(test)]
    pub fn get_connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().await
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


