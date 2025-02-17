#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub name: String,
    pub similarity: f32,
    pub content: String,
    pub chunk_id: usize,
}

#[derive(Serialize, Deserialize)]
pub struct CompletionLogEntry {
    pub timestamp: DateTime<Utc>,
    pub input_text: String,
    pub system_prompt: String,
    pub conversation_context: String,
    pub vector_search_results_for_log: Vec<VectorSearchResult>,
    pub completion_result: String,
}

pub struct Logger {
    log_file: File,
}

impl Logger {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(path)?;

        println!("Opened log file at {}", path);
        Ok(Logger { log_file: file })
    }

    pub fn log_completion(
        &mut self,
        entry: CompletionLogEntry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&entry)?;
        writeln!(self.log_file, "{}", json)?;
        Ok(())
    }
}
