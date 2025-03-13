#![allow(unused_imports)]
#![allow(dead_code)]
// src/embeddings.rs

use async_openai::{
    config::OpenAIConfig, 
    types::CreateEmbeddingRequestArgs, 
    Client,
    error::OpenAIError
};
use reqwest::header::AUTHORIZATION;
use tokio::time::{timeout, sleep};
use std::time::Duration;
use serde_json::json;
use tauri::Emitter;

#[derive(Debug, Clone)]
pub struct EmbeddingGenerator {
    client: Client<OpenAIConfig>,
}

impl EmbeddingGenerator {
    // New constructor no client
    pub fn new() -> Self {
        EmbeddingGenerator { 
            client: Client::new() 
        }
    }
    
    pub fn new_with_client(client: Client<OpenAIConfig>) -> Self {
        EmbeddingGenerator { client: client }
    }
    
    pub fn new_with_api_key(api_key: &str) -> Self {
        let config = OpenAIConfig::new()
        .with_api_key(api_key.to_string());
        let client = Client::with_config(config);
        EmbeddingGenerator { client }
    }
    
    // Optional: Add a constructor that creates a client from an API key
    pub fn from_api_key(api_key: &str) -> Self {
        let config = OpenAIConfig::new()
        .with_api_key(api_key.to_string());
        let client = Client::with_config(config);
        EmbeddingGenerator { client }
    }
    
    pub fn set_api_key(&mut self, api_key: &str) {
        let config = OpenAIConfig::new()
        .with_api_key(api_key.to_string());
        let client = Client::with_config(config);
        self.client = client;
    }
    
    /// Chunks text into segments with optional overlap
    /// 
    /// * `text` - The text to chunk
    /// * `chunk_size` - Maximum size of each chunk in characters
    /// * `overlap` - Number of characters to overlap between chunks
    pub fn chunk_text(&self, text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
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
    
    pub async fn generate_embeddings(
        &self,
        app_handle: tauri::AppHandle,
        text: &str,
        chunk_size: usize,
        overlap: usize,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        let chunks = self.chunk_text(text, chunk_size, overlap);
        let mut embeddings = Vec::new();
        
        for chunk in chunks {
            let embedding = self.generate_embedding(app_handle.clone(), &chunk).await?;
            embeddings.push(embedding);
        }
        
        Ok(embeddings)
    }
    
    pub async fn generate_embedding(
        &self,
        app_handle: tauri::AppHandle,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-ada-002")
        .input(text.to_string())
        .build()?;
        
        let mut retries = 12;
        let mut delay = Duration::from_secs(5);

        while retries > 0 {
            let response = match timeout(Duration::from_secs(60), self.client.embeddings().create(request.clone())).await {
                Ok(result) => {
                    // Handle actual API response or errors
                    match result {
                        Ok(response) => response,
                        Err(err) => {
                            // Check if it's a quota error
                            if let async_openai::error::OpenAIError::ApiError(api_err) = &err {
                                if api_err.code.as_deref() == Some("insufficient_quota") {
                                    let error_message = "OpenAI API quota exceeded. Please check your billing details.";
                                    log::error!("{}: {}", error_message, api_err.message);
                                    app_handle.emit("simple-log-message", json!({
                                        "message": error_message,
                                        "timestamp": chrono::Local::now().to_rfc3339(),
                                        "level": "error"
                                    }))?;
                                    return Err(error_message.into());
                                }
                            }
                            
                            // Generic error handling for other API errors
                            let error_message = format!("OpenAI API error: {}", err);
                            log::error!("{}", error_message);
                            app_handle.emit("simple-log-message", json!({
                                "message": error_message,
                                "timestamp": chrono::Local::now().to_rfc3339(),
                                "level": "error"
                            }))?;
                            return Err(err.into());
                        }
                    }
                },
                Err(_) => {
                    log::error!("OpenAI API call timed out after 60 seconds");
                    app_handle.emit("simple-log-message", json!({
                        "message": format!("OpenAI API call timed out after 60 seconds"),
                        "timestamp": chrono::Local::now().to_rfc3339(),
                        "level": "error"
                    }))?;
                    retries -= 1;
                    if retries > 0 {
                        log::info!("Retrying in {} seconds...", delay.as_secs());
                        sleep(delay).await;
                        delay *= 2; // Exponential backoff
                        continue;
                    } else {
                        return Err("OpenAI API call timed out after 60 seconds".into());
                    }
                }
            };

            if let Some(embedding) = response.data.first() {
                return Ok(embedding.embedding.clone());
            } else {
                return Err("No embedding generated".into());
            }
        }

        Err("Failed to generate embedding after retries".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chunk_text() {
        let generator = EmbeddingGenerator::from_api_key("dummy-key");
        let text = "This is a test text that needs to be chunked into smaller pieces.";
        let chunks = generator.chunk_text(text, 10, 2);
        
        assert!(chunks.len() > 1);
        assert!(chunks[0].len() <= 10);
        
        // Check overlap
        if chunks.len() > 1 {
            let overlap_text = &chunks[0][chunks[0].len()-2..];
            assert!(chunks[1].starts_with(overlap_text));
        }
    }
    
}
