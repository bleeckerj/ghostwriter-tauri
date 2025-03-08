// Declare the modules - these reference the .rs files in this directory
pub mod openai_provider;
pub mod lm_studio_provider;

// Re-export the provider structs so they can be used directly from ai::providers
pub use openai_provider::OpenAIProvider;
pub use lm_studio_provider::LMStudioProvider;

use crate::ai::{
    traits::{ModelProvider, ChatCompletionProvider, EmbeddingProvider, AIProviderError},
    models::*
};
use std::sync::Arc;
use async_trait::async_trait;

/// Enum to represent the type of provider
pub enum ProviderType {
    OpenAI,
    LMStudio,
}

/// Enum to wrap different provider implementations
pub enum Provider {
    OpenAI(OpenAIProvider),
    LMStudio(LMStudioProvider),
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
        }
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
        }
    }

    async fn create_streaming_chat_completion(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<impl futures::Stream<Item = Result<ChatCompletionChunk, AIProviderError>> + Send, AIProviderError> {
        match self {
            Provider::OpenAI(provider) => provider.create_streaming_chat_completion(request).await,
            Provider::LMStudio(provider) => provider.create_streaming_chat_completion(request).await,
        }
    }
}

#[async_trait]
impl ModelProvider for Provider {
    async fn list_models(&self) -> Result<Vec<AIModel>, AIProviderError> {
        match self {
            Provider::OpenAI(provider) => provider.list_models().await,
            Provider::LMStudio(provider) => provider.list_models().await,
        }
    }
    
    async fn get_model(&self, model_id: &str) -> Result<AIModel, AIProviderError> {
        match self {
            Provider::OpenAI(provider) => provider.get_model(model_id).await,
            Provider::LMStudio(provider) => provider.get_model(model_id).await,
        }
    }
}