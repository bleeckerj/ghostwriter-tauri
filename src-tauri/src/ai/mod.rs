pub mod traits;
pub mod models;
pub mod providers;

// Re-export the most important types for convenience
// This lets users write `use crate::ai::AIModel` instead of `use crate::ai::models::AIModel`
pub use traits::{ModelProvider, ChatCompletionProvider, EmbeddingProvider, AIProviderError};
pub use models::{
    AIModel,
    ChatMessage, 
    MessageRole,
    ChatCompletionRequest,
    ChatCompletionResponse
};

// If you want to expose a default provider implementation:
pub use providers::openai_provider::OpenAIProvider;

pub use self::models::*;
pub use self::providers::*;
pub use self::traits::*;