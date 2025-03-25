use async_openai::{Client, config::OpenAIConfig};

use crate::ai::{
    traits::{ModelProvider, ChatCompletionProvider, EmbeddingProvider, PreferredEmbeddingModel, AIProviderError},
    models::*,
};
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;

/// OpenAI implementation of the AI provider traits
pub struct OpenAIProvider {
    client: Client<OpenAIConfig>,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider with the given API key
    pub fn new(api_key: &str) -> Self {
        let config: OpenAIConfig = OpenAIConfig::new().with_api_key(api_key.to_string());
        OpenAIProvider {
            client: Client::with_config(config)
        }
    }
    
    /// Create with an existing OpenAI client
    pub fn with_client(client: Client<OpenAIConfig>) -> Self {
        OpenAIProvider { client }
    }
    
    /// Get a reference to the underlying OpenAI client
    pub fn get_client(&self) -> &Client<OpenAIConfig> {
        &self.client
    }
}

#[async_trait]
impl ModelProvider for OpenAIProvider {
    async fn list_models(&self) -> Result<Vec<AIModel>, AIProviderError> {
        let models = self.client.models().list().await
            .map_err(|e| AIProviderError::APIError(e.to_string()))?;
            
        Ok(models.data.into_iter()
            .map(|m| {
                AIModel {
                    id: m.id.clone(),
                    name: m.id.clone(),
                    provider: "openai".to_string(),
                    capabilities: infer_model_capabilities(&m.id),
                    context_length: infer_context_length(&m.id),
                    additional_info: serde_json::to_value(&m).unwrap_or_default(),
                }
            })
            .collect())
    }
    
    async fn get_model(&self, model_id: &str) -> Result<AIModel, AIProviderError> {
        let model = self.client.models().retrieve(model_id).await
            .map_err(|e| match e.to_string() {
                s if s.contains("404") => AIProviderError::ModelNotAvailable(model_id.to_string()),
                _ => AIProviderError::APIError(e.to_string()),
            })?;
        
        Ok(AIModel {
            id: model.id.clone(),
            name: model.id.clone(),
            provider: "openai".to_string(),
            capabilities: infer_model_capabilities(model_id),
            context_length: infer_context_length(model_id),
            additional_info: serde_json::to_value(&model).unwrap_or_default(),
        })
    }
}

#[async_trait]
impl ChatCompletionProvider for OpenAIProvider {
    async fn create_chat_completion(
        &self, 
        request: &ChatCompletionRequest
    ) -> Result<ChatCompletionResponse, AIProviderError> {
        // Convert to OpenAI specific format
        let openai_messages = convert_messages_to_openai(&request.messages)?;
        
        let openai_request = async_openai::types::CreateChatCompletionRequest {
            model: request.model.clone(),
            messages: openai_messages,
            temperature: request.temperature,
            // Add other parameters as needed
            ..Default::default()
        };
        
        // Make the API call
        let response = self.client.chat().create(openai_request).await
            .map_err(|e| AIProviderError::APIError(e.to_string()))?;
        
        // Convert the response to our generic format
        Ok(convert_openai_completion_response(&response))
    }

    async fn create_streaming_chat_completion(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk, AIProviderError>> + Send>>, AIProviderError> {
        Err(AIProviderError::NotImplemented("Streaming not implemented for OpenAI".to_string()))
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIProvider {
    async fn create_embeddings(
        &self,
        embedding_request: EmbeddingRequest,
        // texts: &[String],
        // model: &str,
    ) -> Result<Vec<Embedding>, AIProviderError> {
        let request = async_openai::types::CreateEmbeddingRequest {
            model: embedding_request.model.to_string(),
            input: async_openai::types::EmbeddingInput::StringArray(embedding_request.input),
            encoding_format: None,
            user: None,
            dimensions: None,
        };
        
        let response = self.client.embeddings().create(request).await
            .map_err(|e| AIProviderError::APIError(e.to_string()))?;
            
        let embedding_model_name = PreferredEmbeddingModel::get_preferred_embedding_model(self);

        Ok(response.data.into_iter()
            .map(|e| Embedding {
                vector: e.embedding,
                index: e.index as usize,
                model_name: Some(embedding_model_name.clone()),
            })
            .collect())
    }
}

impl PreferredEmbeddingModel for OpenAIProvider {
    fn get_preferred_embedding_model(&self) -> String {
        "text-embedding-ada-002".to_string()
    }
}

// Helper functions for model capabilities
fn infer_model_capabilities(model_id: &str) -> Vec<ModelCapability> {
    let mut capabilities = Vec::new();
    
    let model_id = model_id.to_lowercase();
    
    // Add capabilities based on model ID patterns
    if model_id.contains("gpt-3.5") || model_id.contains("gpt-4") {
        capabilities.push(ModelCapability::ChatCompletion);
    }
    
    if model_id.contains("davinci") || model_id.contains("curie") || 
       model_id.contains("babbage") || model_id.contains("ada") {
        capabilities.push(ModelCapability::Completion);
    }
    
    if model_id.contains("text-embedding") || model_id.contains("-e") {
        capabilities.push(ModelCapability::Embedding);
    }
    
    if model_id.contains("dall-e") {
        capabilities.push(ModelCapability::ImageGeneration);
    }
    
    if model_id.contains("whisper") {
        capabilities.push(ModelCapability::AudioTranscription);
    }
    
    if model_id.contains("tts") {
        capabilities.push(ModelCapability::AudioGeneration);
    }
    
    capabilities
}

// Estimate context lengths for common models
fn infer_context_length(model_id: &str) -> Option<usize> {
    let model_id = model_id.to_lowercase();
    
    if model_id.contains("gpt-4-turbo") || model_id.contains("gpt-4-0125") {
        return Some(128000);
    } else if model_id.contains("gpt-4-32k") {
        return Some(32768);
    } else if model_id.contains("gpt-4-preview") || model_id.contains("gpt-4-1106") {
        return Some(128000);
    } else if model_id.contains("gpt-4") {
        return Some(8192);
    } else if model_id.contains("gpt-3.5-turbo-16k") {
        return Some(16384);
    } else if model_id.contains("gpt-3.5-turbo") {
        return Some(4096);
    }
    
    None
}

// Convert our generic messages to OpenAI format
fn convert_messages_to_openai(
    messages: &[ChatMessage]
) -> Result<Vec<async_openai::types::ChatCompletionRequestMessage>, AIProviderError> {
    messages.iter()
        .map(|msg| {
            match msg.role {
                MessageRole::System => {
                    Ok(async_openai::types::ChatCompletionRequestMessage::System(
                        async_openai::types::ChatCompletionRequestSystemMessage {
                            content: async_openai::types::ChatCompletionRequestSystemMessageContent::Text(msg.content.clone()),
                            name: msg.name.clone(),
                        }
                    ))
                },
                MessageRole::User => {
                    Ok(async_openai::types::ChatCompletionRequestMessage::User(
                        async_openai::types::ChatCompletionRequestUserMessage {
                            content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                                msg.content.clone()
                            ),
                            name: msg.name.clone(),
                        }
                    ))
                },
                MessageRole::Assistant => {
                    Ok(async_openai::types::ChatCompletionRequestMessage::Assistant(
                        #[allow(deprecated)]
                        async_openai::types::ChatCompletionRequestAssistantMessage {
                            content: Some(async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(msg.content.clone())),
                            name: msg.name.clone(),
                            tool_calls: None,
                            function_call: None, // Deprecated
                            audio: None,
                            refusal: None,
                        }
                    ))
                },
                MessageRole::Tool | MessageRole::Function => {
                    // For simplicity, not implementing tool messages yet
                    Err(AIProviderError::InvalidRequest(
                        "Tool and Function messages are not supported yet".to_string()
                    ))
                },
            }
        })
        .collect()
}

// Convert OpenAI response to our generic format
#[allow(deprecated)]
fn convert_openai_completion_response(
    response: &async_openai::types::CreateChatCompletionResponse
) -> ChatCompletionResponse {
    ChatCompletionResponse {
        id: response.id.clone(),
        choices: response.choices.iter().map(|choice| {
            let msg = match &choice.message {
                async_openai::types::ChatCompletionResponseMessage { role, content, function_call, tool_calls, .. } => {
                    // Convert the role
                    let role = match role {
                        async_openai::types::Role::System => MessageRole::System,
                        async_openai::types::Role::User => MessageRole::User,
                        async_openai::types::Role::Assistant => MessageRole::Assistant,
                        async_openai::types::Role::Tool => MessageRole::Tool,
                        _ => MessageRole::Assistant, // Default
                    };
                    
                    ChatMessage {
                        role,
                        content: content.clone().unwrap_or_default(),
                        name: None, // OpenAI doesn't return names in responses
                    }
                }
            };
            
            ChatCompletionChoice {
                message: msg,
                finish_reason: choice.finish_reason.clone().map(|e| format!("{:?}", e)),
                index: choice.index as usize,
            }
        }).collect(),
        created: response.created as u64,
        model: response.model.clone(),
        usage: response.usage.as_ref().map(|usage| {
            TokenUsage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            }
        }),
    }
}