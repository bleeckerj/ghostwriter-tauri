// Declare the modules - these reference the .rs files in this directory
pub mod openai_provider;
pub mod lm_studio_provider;
pub mod ollama_provider;

// Re-export the provider structs so they can be used directly from ai::providers
pub use openai_provider::OpenAIProvider;
pub use lm_studio_provider::LMStudioProvider;
pub use ollama_provider::OllamaProvider;

use crate::ai::{
    traits::{ModelProvider, ChatCompletionProvider, EmbeddingProvider, PreferredEmbeddingModel, AIProviderError},
    models::*
};
use std::sync::Arc;
use async_trait::async_trait;

/// Enum to represent the type of provider
pub enum ProviderType {
    OpenAI,
    LMStudio,
    Ollama,
}

/// Enum to wrap different provider implementations
pub enum Provider {
    OpenAI(OpenAIProvider),
    LMStudio(LMStudioProvider),
    Ollama(OllamaProvider),
}

/// Create a provider based on the specified type and configuration
pub fn create_provider(provider_type: ProviderType, config: &str) -> Provider {
    match provider_type {
        ProviderType::OpenAI => {
            Provider::OpenAI(OpenAIProvider::new(config))
        },
        ProviderType::LMStudio => {
            // For LMStudio, use a default URL if empty string is provided
            let url = if config.is_empty() {
                "http://localhost:1234/v1/"  // Default LM Studio URL
            } else {
                config  // Use provided URL if not empty
            };
            Provider::LMStudio(LMStudioProvider::new(url, None))
        },
        ProviderType::Ollama => {
            Provider::Ollama(OllamaProvider::new(config))
        },
    }
}

#[async_trait]
impl ChatCompletionProvider for Provider {
    async fn create_chat_completion(
        &self, 
        request: &ChatCompletionRequest
    ) -> Result<ChatCompletionResponse, AIProviderError> {
        match self {
            Provider::OpenAI(provider) => provider.create_chat_completion(request).await,
            Provider::LMStudio(provider) => provider.create_chat_completion(request).await,
            Provider::Ollama(provider) => provider.create_chat_completion(request).await,
        }
    }

    async fn create_streaming_chat_completion(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<impl futures::Stream<Item = Result<ChatCompletionChunk, AIProviderError>> + Send, AIProviderError> {
        match self {
            Provider::OpenAI(provider) => provider.create_streaming_chat_completion(request).await,
            Provider::LMStudio(provider) => provider.create_streaming_chat_completion(request).await,
            Provider::Ollama(provider) => provider.create_streaming_chat_completion(request).await,
        }
    }
}

#[async_trait]
impl EmbeddingProvider for Provider {
    async fn create_embeddings(
        &self,
        embedding_request: EmbeddingRequest,
    ) -> Result<Vec<Embedding>, AIProviderError> {
        match self {
            Provider::OpenAI(provider) => provider.create_embeddings(embedding_request).await,
            Provider::LMStudio(provider) => provider.create_embeddings(embedding_request).await,
            Provider::Ollama(provider) => provider.create_embeddings(embedding_request).await,
        }
    }
}

#[async_trait]
impl ModelProvider for Provider {
    async fn list_models(&self) -> Result<Vec<AIModel>, AIProviderError> {
        match self {
            Provider::OpenAI(provider) => provider.list_models().await,
            Provider::LMStudio(provider) => provider.list_models().await,
            Provider::Ollama(provider) => provider.list_models().await,
        }
    }
    
    async fn get_model(&self, model_id: &str) -> Result<AIModel, AIProviderError> {
        match self {
            Provider::OpenAI(provider) => provider.get_model(model_id).await,
            Provider::LMStudio(provider) => provider.get_model(model_id).await,
            Provider::Ollama(provider) => provider.get_model(model_id).await,
        }
    }
}

impl PreferredEmbeddingModel for Provider {
    fn get_preferred_embedding_model(&self) -> String {
        match self {
            Provider::OpenAI(provider) => provider.get_preferred_embedding_model(),
            Provider::LMStudio(provider) => provider.get_preferred_embedding_model(),
            Provider::Ollama(provider) => provider.get_preferred_embedding_model(),
        }
    }
}