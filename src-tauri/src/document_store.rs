#![allow(unused_imports)]
#![allow(unused)]

// src/document_store.rs
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: usize,
    pub content: String,
    pub path: String,
    pub embedding: Vec<f32>,
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
            "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                content TEXT NOT NULL,
                path TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS embeddings (
                doc_id INTEGER PRIMARY KEY,
                vector BLOB NOT NULL,
                FOREIGN KEY(doc_id) REFERENCES documents(id)
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

        let tx = self.conn.transaction()?;

        // Insert document
        tx.execute(
            "INSERT INTO documents (id, content, path) VALUES (?1, ?2, ?3)",
            params![document.id, document.content, document.path],
        )?;

        // Insert embedding
        let embedding_bytes = bincode::serialize(&document.embedding)?;
        tx.execute(
            "INSERT INTO embeddings (doc_id, vector) VALUES (?1, ?2)",
            params![document.id, embedding_bytes],
        )?;

        tx.commit()?;

        self.next_id += 1;
        Ok(())
    }

    pub fn search(
        &self,
        query: &[f32],
        limit: usize,
    ) -> Result<Vec<(Document, f32)>, Box<dyn std::error::Error>> {
        let mut documents = Vec::new();

        let mut stmt = self.conn.prepare(
            "SELECT d.id, d.content, d.path, e.vector 
             FROM documents d 
             JOIN embeddings e ON d.id = e.doc_id",
        )?;

        let rows = stmt.query_map([], |row| {
            let id: usize = row.get(0)?;
            let content: String = row.get(1)?;
            let path: String = row.get(2)?;
            let vector_bytes: Vec<u8> = row.get(3)?;

            let embedding: Vec<f32> = bincode::deserialize(&vector_bytes).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Blob,
                    Box::new(e),
                )
            })?;

            Ok(Document {
                id,
                content,
                path,
                embedding,
            })
        })?;

        // Collect documents and compute similarities
        let mut similarities: Vec<(Document, f32)> = Vec::new();
        for row in rows {
            let doc = row?;
            let similarity = cosine_similarity(query, &doc.embedding);
            similarities.push((doc, similarity));
        }

        // Sort by similarity and take top k
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        documents.extend(similarities.into_iter().take(limit));

        Ok(documents)
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
