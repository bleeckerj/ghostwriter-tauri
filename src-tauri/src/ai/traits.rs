use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use crate::ai::models::AIModel;
use crate::ai::models::ChatCompletionRequest;
use crate::ai::models::ChatCompletionResponse;
use crate::ai::models::ChatCompletionChunk;
use crate::ai::models::Embedding;
use crate::ai::models::EmbeddingRequest;

/// Represents any error that can occur when interacting with AI providers
#[derive(Debug, thiserror::Error)]
pub enum AIProviderError {
    #[error("API error: {0}")]
    APIError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Authentication failure: {0}")]
    AuthError(String),
    
    #[error("Model not available: {0}")]
    ModelNotAvailable(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Generic error: {0}")]
    Other(String),

    #[error("Not implemented error: {0}")]
    NotImplemented(String),

    #[error("Model not found error: {0}")]
    ModelNotFound(String),
    
    #[error("Deserialization error: {0}")]  
    DeserializationError(String),
}

/// Core trait for retrieving models
#[async_trait]
pub trait ModelProvider {
    /// List available models
    async fn list_models(&self) -> Result<Vec<AIModel>, AIProviderError>;
    
    /// Get information about a specific model
    async fn get_model(&self, model_id: &str) -> Result<AIModel, AIProviderError>;

    // Get the preferred model to use for inference
    async fn get_preferred_inference_model(&self, preference_model: &str) -> Result<AIModel, AIProviderError>;

    fn get_provider_name(&self) -> String;

    fn set_preferred_inference_model(&mut self, model_name: String) -> Result<(), AIProviderError>;
}

/// Core trait for chat completions
#[async_trait]
pub trait ChatCompletionProvider {
    /// Generate chat completion
    async fn create_chat_completion(
        &self, 
        request: &ChatCompletionRequest
    ) -> Result<ChatCompletionResponse, AIProviderError>;
    
    /// Stream chat completions
    async fn create_streaming_chat_completion(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<impl futures::Stream<Item = Result<ChatCompletionChunk, AIProviderError>> + Send, AIProviderError>;
}

/// Core trait for embeddings
#[async_trait]
pub trait EmbeddingProvider {
    /// Generate embeddings for text
    async fn create_embeddings(
        &self,
        // texts: &[String],
        // model: &str,
        embedding_request: EmbeddingRequest,
    ) -> Result<Vec<Embedding>, AIProviderError>;
}

/// Trait to get the preferred embedding model
pub trait PreferredEmbeddingModel {
    fn get_preferred_embedding_model(&self) -> String;
}

#[async_trait]
pub trait DiffusingProvider {
    async fn create_diffusing_stream(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<impl futures::Stream<Item = Result<ChatCompletionChunk, AIProviderError>> + Send, AIProviderError>;
}