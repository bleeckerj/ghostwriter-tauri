#![allow(unused_imports)]
#![allow(dead_code)]
// src/embeddings.rs
use async_openai::{config::OpenAIConfig, types::CreateEmbeddingRequestArgs, Client};

pub struct EmbeddingGenerator {
    client: Client<OpenAIConfig>,
}

impl EmbeddingGenerator {
    pub fn new() -> Self {
        EmbeddingGenerator {
            client: Client::new(),
        }

        // // OR use API key from different source and a non default organization
        // let api_key = "sk-..."; // This secret could be from a file, or environment variable.
        // let config = OpenAIConfig::new()
        //     .with_api_key(api_key)
        //     .with_org_id("the-continental");
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
