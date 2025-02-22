#![allow(unused_imports)]
#![allow(dead_code)]
// src/embeddings.rs

use async_openai::{config::OpenAIConfig, types::CreateEmbeddingRequestArgs, Client};
#[derive(Clone)]
pub struct EmbeddingGenerator {
    client: Client<OpenAIConfig>,
}

impl EmbeddingGenerator {
    // New constructor that takes a client
    pub fn new(client: Client<OpenAIConfig>) -> Self {
        EmbeddingGenerator { client }
    }

    // Optional: Add a constructor that creates a client from an API key
    pub fn from_api_key(api_key: &str) -> Self {
        let config = OpenAIConfig::new()
            .with_api_key(api_key.to_string());
        let client = Client::with_config(config);
        EmbeddingGenerator { client }
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
        text: &str,
        chunk_size: usize,
        overlap: usize,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        let chunks = self.chunk_text(text, chunk_size, overlap);
        let mut embeddings = Vec::new();

        for chunk in chunks {
            let embedding = self.generate_embedding(&chunk).await?;
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    pub async fn generate_embedding(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model("text-embedding-ada-002")
            .input(text.to_string())
            .build()?;

        let response = self.client.embeddings().create(request).await?;

        if let Some(embedding) = response.data.first() {
            Ok(embedding.embedding.clone())
        } else {
            Err("No embedding generated".into())
        }
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
