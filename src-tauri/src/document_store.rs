#![allow(unused_imports)]
#![allow(unused)]

// src/document_store.rs
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::Local;  // Add this to your imports at the top
use serde_json;

use crate::embeddings;  // First, add serde_json to your imports

#[derive(Clone, Serialize, Deserialize)]

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
    conn: Connection,
    // Keep track of next ID since we're not using autoincrement
    next_id: usize,
}

impl DocumentStore {
    pub fn new(store_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&store_path)?;
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

        Ok(DocumentStore { conn, next_id })
    }

    pub fn add_document(
        &mut self,
        mut document: Document,
    ) -> Result<(), Box<dyn std::error::Error>> {
        document.id = self.next_id;
        let current_time = Local::now().to_rfc3339();  // ISO 8601/RFC 3339 format

        let tx = self.conn.transaction()?;

        // Insert document
        tx.execute(
            "INSERT INTO documents (id, name, created_at, file_path) VALUES (?1, ?2, ?3, ?4)",
            params![document.id, document.name, current_time, document.file_path],
        )?;

        // // Insert embedding
        // let embedding_bytes = bincode::serialize(&document.embedding)?;
        // tx.execute(
        //     "INSERT INTO embeddings (doc_id, chunk, embedding) VALUES (?, ?, ?)",
        //     params![document.id, chunk, embedding_bytes],
        // )?;

        tx.commit()?;

        self.next_id += 1;
        Ok(())
    }

    pub fn search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, usize, String, f32)>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
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

    pub fn fetch_documents(&self) -> Result<DocumentListing, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare("SELECT id, name, file_path, created_at FROM documents")?;
        
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
        let db_path = self.conn.path().unwrap_or_default();
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
