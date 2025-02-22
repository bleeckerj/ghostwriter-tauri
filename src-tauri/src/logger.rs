#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::io::{self, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use chrono::Local;

const MAX_ENTRIES: usize = 3;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VectorSearchResult {
    pub name: String,
    pub similarity: f32,
    pub content: String,
    pub chunk_id: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompletionLogEntry {
    pub timestamp: DateTime<Utc>,
    pub input_text: String,
    pub completion_result: String,
    pub system_prompt: String,
    pub conversation_context: String,
    pub vector_search_results_for_log: Vec<VectorSearchResult>,
    pub canon_name: String,
    pub canon_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Completion {
    pub completion: CompletionLogEntry,
}

pub struct Logger {
    log_file: File,
    log_path: PathBuf,
}

impl Logger {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let log_path = PathBuf::from(path);
        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&log_path)?;

        // Initialize empty array if file is empty
        let metadata = file.metadata()?;
        if metadata.len() == 0 {
            serde_json::to_writer(&file, &Vec::<Completion>::new())?;
        }

        println!("Opened log file at {}", path);
        Ok(Logger { log_file: file, log_path })
    }

    fn rotate_log_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Generate timestamp for the backup file
        let timestamp = Local::now().format("%m%d%y_%H%M%S");
        let parent = self.log_path.parent().unwrap_or_else(|| Path::new(""));
        let file_stem = self.log_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("log");
        let extension = self.log_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("json");
        
        // Create backup file name with timestamp
        let backup_path = parent.join(format!("{}_{}.{}", file_stem, timestamp, extension));
        
        // Close current file
        self.log_file.flush()?;
        let _ = &self.log_file;

        // Rename current file to backup name
        std::fs::rename(&self.log_path, &backup_path)?;

        // Create new empty log file
        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&self.log_path)?;
        
        // Initialize with empty array
        serde_json::to_writer(&file, &Vec::<Completion>::new())?;
        
        self.log_file = file;
        Ok(())
    }

    pub fn log_completion(
        &mut self,
        entry: Completion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Read existing entries
        self.log_file.seek(SeekFrom::Start(0))?;
    
        let entries: Vec<Completion> = match serde_json::from_reader(BufReader::new(&self.log_file)) {
            Ok(entries) => entries,
            Err(e) => {
                println!("Failed to read existing entries, initializing new vector: {}", e);
                Vec::new()
            }
        };
        
        let mut updated_entries = entries;
        updated_entries.push(entry);

        // Check if we need to rotate
        if updated_entries.len() > MAX_ENTRIES {
            self.rotate_log_file()?;
            updated_entries = vec![updated_entries.last().unwrap().clone()];
        }
    
        // Write back all entries
        self.log_file.seek(SeekFrom::Start(0))?;
        self.log_file.set_len(0)?;
        serde_json::to_writer_pretty(&self.log_file, &updated_entries)?;
        self.log_file.flush()?;
    
        Ok(())
    }
}
