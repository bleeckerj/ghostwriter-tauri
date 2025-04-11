use crate::{ai::{
    models::*, traits::{AIProviderError, ChatCompletionProvider, EmbeddingProvider, ModelProvider, PreferredEmbeddingModel}
}, preferences::Preferences};

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use futures::{Stream, StreamExt};
use std::pin::Pin;
use ollama_rs::{
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage}, 
        embeddings::request::EmbeddingsInput,
        embeddings::request::GenerateEmbeddingsRequest,
    },
    Ollama,
    generation::completion::request::GenerationRequest,
    generation::options::GenerationOptions,
};


use tokio::io::{stdout, AsyncWriteExt};

/// Provider implementation for Ollama API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaProvider {
    #[serde(skip)]
    client: Ollama,
    preferred_model_name: Option<String>,
}

impl OllamaProvider {
    pub fn new(url: &str) -> Self {
        // Use default URL if empty string is provided
        let ollama = if url.is_empty() {
            Ollama::default()
        } else {
            // Parse the URL to extract host and port
            if let Ok(parsed_url) = url::Url::parse(url) {
                let host = parsed_url.host_str().unwrap_or("localhost").to_string();
                let port = parsed_url.port().unwrap_or(11434);
                
                
                // Try to create a URL from components to catch potential errors
                if let Ok(url) = url::Url::parse(&format!("http://{}:{}", host, port)) {
                    // Use from_url which doesn't have the unwrap calls
                    Ollama::from_url(url)
                } else {
                    Ollama::default()
                }
            } else {
                log::warn!("Ollama initialization failed to parse URL: {}", url);
                log::warn!("We'll be using the default URL instead");
                Ollama::default()
            }
        };
        
        OllamaProvider {
            client: ollama,
            preferred_model_name: None,
        }
    }
}

#[async_trait]
impl ModelProvider for OllamaProvider {
    async fn list_models(&self) -> Result<Vec<AIModel>, AIProviderError> {
        // Use the actual ollama-rs API
        let response = self.client.list_local_models().await
        .map_err(|e| AIProviderError::APIError(format!("Failed to list models: {}", e)))?;
        
        // Convert to our model format
        let models = response
        .into_iter()
        .map(|m| AIModel {
            id: m.name.clone(),
            name: m.name,
            provider: "ollama".to_string(),
            capabilities: vec![
            ModelCapability::ChatCompletion,
            ModelCapability::Embedding,
            ],
            context_length: None,
            additional_info: serde_json::Value::Null,
        })
        .collect();
        
        Ok(models)
    }
    
    async fn get_model(&self, model_id: &str) -> Result<AIModel, AIProviderError> {
        // Get the list of models and find the requested one
        let models = self.client.list_local_models().await
        .map_err(|e| AIProviderError::APIError(format!("Failed to list models: {}", e)))?;
        
        let model = models.into_iter()
        .find(|m| m.name == model_id)
        .ok_or_else(|| AIProviderError::ModelNotAvailable(model_id.to_string()))?;
        
        Ok(AIModel {
            id: model.name.clone(),
            name: model.name,
            provider: "ollama".to_string(),
            capabilities: vec![
            ModelCapability::ChatCompletion,
            ModelCapability::Embedding,
            ],
            context_length: None,
            additional_info: serde_json::Value::Null,
        })
    }

    async fn get_preferred_inference_model(&self, preference_model: &str) -> Result<AIModel, AIProviderError> {
        let all_models = self.list_models().await
            .map_err(|e| AIProviderError::APIError(format!("Failed to list models: {}", e)))?;
        
        // First try to find the requested model
        for model in &all_models {
            if model.name == preference_model {
                log::info!("Using requested model: {}", model.name);
                return Ok(model.clone());
            }
        }
        
        // If requested model not found, try llama3.2:latest (existing fallback)
        for model in &all_models {
            if model.name == "llama3.2:latest" {
                log::info!("Requested model '{}' not found, using llama3.2:latest", preference_model);
                return Ok(model.clone());
            }
        }
        
        // If llama3.2 not found, try other common models
        for candidate in &["llama3.2:latest", "llama2:latest", "mistral:latest"] {
            if let Some(model) = all_models.iter().find(|m| m.name == *candidate) {
                log::info!("Using fallback model: {}", model.name);
                return Ok(model.clone());
            }
        }
        
        // Last resort - use first available model
        if let Some(first_model) = all_models.first() {
            log::warn!("No preferred models found, using first available model: {}", first_model.name);
            return Ok(first_model.clone());
        }
        
        // Complete fallback - create a default model object
        // This will only be used if no models are available at all
        log::error!("No models available, returning default model configuration");
        Ok(AIModel {
            id: "llama3.2:latest".to_string(),
            name: "llama3.2:latest".to_string(),
            provider: "ollama".to_string(),
            capabilities: vec![
                ModelCapability::ChatCompletion,
                ModelCapability::Embedding,
            ],
            context_length: None,
            additional_info: serde_json::Value::Null,
        })
    }

    fn set_preferred_inference_model(&mut self, model_name: String) -> Result<(), AIProviderError> {
            // Set the preferred model
            self.preferred_model_name = Some(model_name);
            Ok(())
    }

    fn get_provider_name(&self) -> String {
        "Ollama".to_string()
    }
        
}

#[async_trait]
impl ChatCompletionProvider for OllamaProvider {
    async fn create_chat_completion(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AIProviderError> {
        // Combine messages into a simple prompt as in the example
        let messages = convert_messages(&request.messages);
        //let ollama = Ollama::default();
        let model = request.model.clone();
        //let __prompt = "<system>You are just a text completion engine. Do not precede your response with '...'</system>When the core reciprocating GPU-based mechanical-electrical engine connected to the network, we found that the farm operations were ".to_string();
        let prompt = messages_to_prompt(&request.messages);
        //let temp = Preferences::load(app_state, app_handle)
        let temp = request.temperature.unwrap_or(0.2);
        let max_tokens = request.max_tokens.unwrap_or(10) as i32;
        let options = GenerationOptions::default()
        .temperature(temp)
        .repeat_penalty(1.5)
        .top_k(25)
        .num_predict(max_tokens)
        .top_p(0.25);
    
        let request = GenerationRequest::new(model, prompt.clone()).options(options);
        println!("***Request: {:?}", request.clone());
        println!("***Prompt: {:?}", prompt.clone());
        let res = self.client
            .generate(request)
            .await
            .map_err(|e| AIProviderError::APIError(format!("Failed to generate generate response /api/generate completion: {}", e)))?;

        // let res = ollama
        //     .generate(GenerationRequest::new(model, prompt).options(options))
        //     .await;
    
        println!("Response: {:?}", res);

        Ok(ChatCompletionResponse {
            id: uuid::Uuid::new_v4().to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: res.model,
            choices: vec![ChatCompletionChoice {
                index: 0,
                message: crate::ai::models::ChatMessage {
                    role: MessageRole::Assistant,
                    content: res.response,
                    name: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
        })

        // OLD OLD OLD using /api/chat
        // Create and send the request
        // let chat_request = ChatMessageRequest::new(request.model.clone(), messages);
        // let response = self.client.send_chat_messages(chat_request).await
        // .map_err(|e| AIProviderError::APIError(format!("Chat completion failed: {}", e)))?;
        
        // Ok(ChatCompletionResponse {
        //     id: uuid::Uuid::new_v4().to_string(),
        //     created: chrono::Utc::now().timestamp() as u64,
        //     model: request.model.clone(),
        //     choices: vec![ChatCompletionChoice {
        //         index: 0,
        //         message: crate::ai::models::ChatMessage {
        //             role: MessageRole::Assistant,
        //             content: response.message.content,
        //             name: None,
        //         },
        //         finish_reason: Some("stop".to_string()),
        //     }],
        //     usage: None,
        // })
    }
    
    async fn create_streaming_chat_completion(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk, AIProviderError>> + Send>>, AIProviderError> {
        // Combine messages into a simple prompt
        Err(AIProviderError::NotImplemented("Streaming not implemented for Ollama".to_string()))
    }
}

fn convert_messages(messages: &[crate::ai::models::ChatMessage]) -> Vec<ChatMessage> {
    messages
    .iter()
    .map(|msg| {
        match msg.role {
            MessageRole::User => ChatMessage::user(msg.content.clone()),
            MessageRole::Assistant => ChatMessage::assistant(msg.content.clone()),
            MessageRole::System => ChatMessage::system(msg.content.clone()),
            MessageRole::Tool => ChatMessage::user(msg.content.clone()), // Fallback for tool
            MessageRole::Function => ChatMessage::user(msg.content.clone()), // Fallback for function
        }
    })
    .collect()
}

fn convert_ollama_message(msg: &ollama_rs::generation::chat::ChatMessage) -> crate::ai::models::ChatMessage {
    let role = match msg.role {
        ollama_rs::generation::chat::MessageRole::User => MessageRole::User,
        ollama_rs::generation::chat::MessageRole::Assistant => MessageRole::Assistant,
        ollama_rs::generation::chat::MessageRole::System => MessageRole::System,
        ollama_rs::generation::chat::MessageRole::Tool => MessageRole::Tool,
    };
    
    crate::ai::models::ChatMessage {
        role,
        content: msg.content.clone(),
        name: None, // Ollama doesn't provide a name
    }
}

/// Converts a list of chat messages to a formatted string prompt
fn messages_to_prompt(messages: &[crate::ai::models::ChatMessage]) -> String {
    let mut prompt = String::new();
    
    for message in messages {
        match message.role {
            MessageRole::System => {
                // System messages are typically wrapped in special tags
                prompt.push_str(&format!("<system>{}</system>\n", message.content));
            },
            MessageRole::User => {
                // Format user messages
                prompt.push_str(&format!("<user>{}</user>\n", message.content));
            },
            MessageRole::Assistant => {
                // Format assistant responses
                //prompt.push_str(&format!("<assistant>{}</assistant>\n", message.content));
            },
            MessageRole::Tool => {
                // Handle tool/function messages as generic content with a tag
                prompt.push_str(&format!("<{role}>{content}</{role}>\n", 
                    role = "tool".to_string(),
                    content = message.content));
            },
            MessageRole::Function => {
                // Handle tool/function messages as generic content with a tag
                prompt.push_str(&format!("<{role}>{content}</{role}>\n", 
                    role = "function".to_string(),
                    content = message.content));
            }
        }
    }
    
    // Add a final assistant prefix to prompt the model to continue
    if !messages.is_empty() && messages.last().unwrap().role != MessageRole::Assistant {
        prompt.push_str("Assistant: ");
    }
    
    prompt
}

#[async_trait]
impl EmbeddingProvider for OllamaProvider {
    async fn create_embeddings(
        &self,
        embedding_request: EmbeddingRequest,
    ) -> Result<Vec<Embedding>, AIProviderError> {

        let mut embeddings: Vec<Embedding> = Vec::new();
        let input = EmbeddingsInput::Multiple(embedding_request.input.clone());
        
        // Create a request to generate embeddings
        let request = GenerateEmbeddingsRequest::new(
            embedding_request.model.to_string(),
            input,
        );
        
        let response = self.client.generate_embeddings(request).await
        .map_err(|e| AIProviderError::APIError(format!("Embedding failed: {}", e)))?;
        
        let embedding_model_name = PreferredEmbeddingModel::get_preferred_embedding_model(self);

        // Extract the embeddings from the response
        let embeddings = response.embeddings.into_iter()
        .enumerate()
        .map(|(index, vector)| Embedding {
            vector,
            index,
            model_name: Some(embedding_model_name.clone()),
        })
        .collect();
        //log::debug!("Embeddings: {:?}", embeddings);
        //println!("Embeddings: {:?}", embeddings);
        Ok(embeddings)
    }
}

impl PreferredEmbeddingModel for OllamaProvider {
    fn get_preferred_embedding_model(&self) -> String {
        "nomic-embed-text".to_string()
        //unimplemented!("Ollama does not yet support a preferred embedding model");
    }
}