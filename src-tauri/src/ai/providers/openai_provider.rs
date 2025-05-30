#![allow(deprecated)]
use async_openai::{Client, config::OpenAIConfig};
use serde::{Serialize, Deserialize, ser::SerializeStruct, de::{self, Deserializer, Visitor, MapAccess}};
use std::fmt;

use crate::ai::{
    traits::{ModelProvider, ChatCompletionProvider, EmbeddingProvider, PreferredEmbeddingModel, AIProviderError},
    models::*,
};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use async_openai::types::CreateChatCompletionRequest;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref LAST_REQUEST: Mutex<Option<CreateChatCompletionRequest>> = Mutex::new(None);
}

/// OpenAI implementation of the AI provider traits
#[derive(Debug, Clone, Serialize)]
pub struct OpenAIProvider {
    #[serde(skip)]
    client: Client<OpenAIConfig>,
    #[serde(skip)]
    last_request: Option<CreateChatCompletionRequest>,
    preferred_model_name: Option<String>,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider with the given API key
    pub fn new(api_key: &str) -> Self {
        let config: OpenAIConfig = OpenAIConfig::new().with_api_key(api_key.to_string());
        OpenAIProvider {
            client: Client::with_config(config),
            last_request: None,
            preferred_model_name: None,
        }
    }
    
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {

        let mut state = serializer.serialize_struct("OpenAIProvider", 1)?;
        state.serialize_field("provider", &"openai_provider")?;
        state.end()
    }

    /// Create with an existing OpenAI client
    pub fn with_client(client: Client<OpenAIConfig>) -> Self {
        OpenAIProvider { client, last_request: None, preferred_model_name: None }
    }
    
    /// Get a reference to the underlying OpenAI client
    pub fn get_client(&self) -> &Client<OpenAIConfig> {
        &self.client
    }
}

impl OpenAIProvider {

    fn set_last_request(request: CreateChatCompletionRequest) {
        let mut last_request = LAST_REQUEST.lock().unwrap();
        *last_request = Some(request);
    }

    fn get_last_request() -> Option<CreateChatCompletionRequest> {
        let last_request = LAST_REQUEST.lock().unwrap();
        last_request.clone()
    }
}

impl<'de> Deserialize<'de> for OpenAIProvider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Define a visitor to handle deserialization
        struct OpenAIProviderVisitor;

        impl<'de> Visitor<'de> for OpenAIProviderVisitor {
            type Value = OpenAIProvider;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct OpenAIProvider")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut preferred_model_name = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "preferred_model_name" => {
                            preferred_model_name = map.next_value()?;
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?; // Ignore unknown fields
                        }
                    }
                }

                Ok(OpenAIProvider {
                    client: Client::with_config(OpenAIConfig::new()), // Default client
                    last_request: None, // Default value
                    preferred_model_name,
                })
            }
        }

        // Use the visitor to deserialize the struct
        deserializer.deserialize_struct(
            "OpenAIProvider",
            &["preferred_model_name"],
            OpenAIProviderVisitor,
        )
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

    async fn get_preferred_inference_model(&self, preference_model: &str) -> Result<AIModel, AIProviderError> {
        let all_models = self.list_models().await?;

        // Try to find the preferred model
        for model in &all_models {
            if model.name == preference_model {
                log::info!("Using preference model: {}", model.name);
                return Ok(model.clone());
            }
        }
        
        // Look for a sensible default model (gpt-4o-mini, gpt-3.5-turbo, etc.)
        let default_candidates = ["gpt-4o-mini", "gpt-3.5-turbo", "gpt-4"];
        
        for candidate in default_candidates {
            if let Some(model) = all_models.iter().find(|m| m.name.contains(candidate)) {
                log::warn!("Using default model: {}", model.name);
                return Ok(model.clone());
            }
        }
        
        // If all else fails, create a hardcoded default model
        Ok(AIModel {
            id: "gpt-4o-mini".to_string(),
            name: "gpt-4o-mini".to_string(),
            provider: "openai".to_string(),
            capabilities: vec![ModelCapability::ChatCompletion],
            context_length: Some(4096),
            additional_info: serde_json::json!({}),
        })
    }

    fn set_preferred_inference_model(&mut self, model_name: String) -> Result<(), AIProviderError> {
        // Set the preferred model
        self.preferred_model_name = Some(model_name.clone());
        Ok(())
    }

    fn get_provider_name(&self) -> String {
        "openai".to_string()
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
        
        let openai_request = CreateChatCompletionRequest {
            model: request.model.clone(),
            messages: openai_messages,
            temperature: request.temperature,
            // Add other parameters as needed
            ..Default::default()
        };

        // Store the last request in the global state
        //OpenAIProvider::set_last_request(openai_request.clone());
        
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
        // Convert to OpenAI specific format
        let openai_messages = convert_messages_to_openai(&request.messages)?;
        
        let openai_request = CreateChatCompletionRequest {
            model: request.model.clone(),
            messages: openai_messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: Some(true),
            ..Default::default()
        };

        // Make the API call with streaming
        let stream = self.client.chat().create_stream(openai_request).await
            .map_err(|e| AIProviderError::APIError(e.to_string()))?;
        
        // Map the OpenAI stream to our generic format
        let mapped_stream = StreamExt::map(stream, move |result| match result {
            Ok(response) => {
                // Convert OpenAI response chunk to our generic format
                let choices = response.choices.iter()
                    .map(|choice| {
                        ChatCompletionChunkChoice {
                            index: choice.index as usize,
                            delta: ChatMessageDelta {
                                role: None, // Roles typically come in the first chunk only
                                content: choice.delta.content.clone(),
                            },
                            finish_reason: choice.finish_reason.clone().map(|r| format!("{:?}", r)),
                        }
                    })
                    .collect();

                Ok(ChatCompletionChunk {
                    id: response.id.clone(),
                    created: response.created as u64,
                    choices,
                })
            },
            Err(e) => Err(AIProviderError::APIError(e.to_string())),
        });

        Ok(Box::pin(mapped_stream))
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