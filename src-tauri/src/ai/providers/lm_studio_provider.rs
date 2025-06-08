use crate::ai::{
    traits::{ModelProvider, ChatCompletionProvider, EmbeddingProvider, PreferredEmbeddingModel, AIProviderError},
    models::*,
};
use async_trait::async_trait;
use reqwest::{Client as HttpClient, header};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};
use futures::{stream, Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use lazy_static::lazy_static;
use std::sync::Mutex;
use async_openai::types::CreateChatCompletionRequest;

lazy_static! {
    static ref LAST_REQUEST: Mutex<Option<CreateChatCompletionRequest>> = Mutex::new(None);
}

impl LMStudioProvider {
    fn set_last_request(request: CreateChatCompletionRequest) {
        let mut last_request = LAST_REQUEST.lock().unwrap();
        *last_request = Some(request);
    }

    fn get_last_request() -> Option<CreateChatCompletionRequest> {
        let last_request = LAST_REQUEST.lock().unwrap();
        last_request.clone()
    }
}
/// LM Studio provider implementation

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LMStudioProvider {
    #[serde(skip)] 
    client: HttpClient,
    base_url: String,
    api_key: Option<String>,
    preferred_model_name: Option<String>,
}

impl LMStudioProvider {
    /// Create a new LM Studio provider with the specified base URL
    pub fn new(base_url: &str, api_key: Option<String>) -> Self {
        let mut headers = header::HeaderMap::new();
        
        // Add content-type header
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        // Create client with default configuration
        let client = HttpClient::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        LMStudioProvider {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            preferred_model_name: None,
        }
    }

    /// Helper method to add authorization header if API key is set
    fn add_auth_header(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.api_key {
            Some(key) if !key.is_empty() => {
                builder.header("Authorization", format!("Bearer {}", key))
            },
            _ => builder,
        }
    }
}

#[async_trait]
impl ModelProvider for LMStudioProvider {
    async fn list_models(&self) -> Result<Vec<AIModel>, AIProviderError> {
        let url = format!("{}/models", self.base_url);
        
        let request = self.client.get(&url);
        //let request = self.add_auth_header(request);
        println!("Request URL: {}", url);
        println!("Request {:?}", request);

        let response = request.send().await
            .map_err(|e| AIProviderError::APIError(format!("Network error: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
            log::debug!("Failed to list models: {}: {}", status, text);  
            return Err(AIProviderError::APIError(format!(
                "API returned error {}: {}", status, text
            )));
        }
        
        #[derive(Deserialize)]
        #[derive(Serialize)]
        struct ModelData {
            id: String,
            object: String,
            created: Option<u64>,
            owned_by: Option<String>,
        }
        
        #[derive(Deserialize)]
        struct ModelListResponse {
            data: Vec<ModelData>,
            object: String,
        }
        
        let model_list: ModelListResponse = response.json().await
            .map_err(|e| AIProviderError::APIError(format!("Failed to parse response: {}", e)))?;
            
        Ok(model_list.data.into_iter()
            .map(|m| AIModel {
                id: m.id.clone(),
                name: m.id.clone(),
                provider: "lm_studio".to_string(),
                capabilities: vec![ModelCapability::ChatCompletion], // Most LM Studio models support chat
                context_length: None, // Not provided by the API
                additional_info: serde_json::to_value(m).unwrap_or_default(),
            })
            .collect())
    }
    
    async fn get_model(&self, model_id: &str) -> Result<AIModel, AIProviderError> {
        let url = format!("{}/models/{}", self.base_url, model_id);
        
        let request = self.client.get(&url);
        let request = self.add_auth_header(request);
        println!("Request URL: {}", url);
        println!("Request {:?}", request);


        let response = request.send().await
            .map_err(|e| AIProviderError::APIError(format!("Network error: {}", e)))?;
            
        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Err(AIProviderError::ModelNotAvailable(model_id.to_string()));
            }
            
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
                
            return Err(AIProviderError::APIError(format!(
                "API returned error {}: {}", status, text
            )));
        }
        
        let model_data: Value = response.json().await
            .map_err(|e| AIProviderError::APIError(format!("Failed to parse response: {}", e)))?;
            
        Ok(AIModel {
            id: model_data["id"].as_str().unwrap_or(model_id).to_string(),
            name: model_data["id"].as_str().unwrap_or(model_id).to_string(),
            provider: "lm_studio".to_string(),
            capabilities: vec![ModelCapability::ChatCompletion], // Most LM Studio models support chat
            context_length: None, // Not provided by the API
            additional_info: model_data,
        })
    }

    async fn get_preferred_inference_model(&self, preference_model: &str) -> Result<AIModel, AIProviderError> {
        // Use the preferred_model_name if set, otherwise use the provided preference_model
        let model_id = match &self.preferred_model_name {
            Some(model) => model.clone(),
            None => preference_model.to_string(),
        };
        
        // Try to fetch the model info
        match self.get_model(&model_id).await {
            Ok(model) => Ok(model),
            Err(AIProviderError::ModelNotAvailable(_)) => {
                // If model isn't available, try to get the first available model
                let models = self.list_models().await?;
                if let Some(first_model) = models.first() {
                    Ok(first_model.clone())
                } else {
                    Err(AIProviderError::ModelNotAvailable(format!(
                        "No models available for LM Studio provider"
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set_preferred_inference_model(&mut self, model_name: String) -> Result<(), AIProviderError> {
        self.preferred_model_name = Some(model_name);
        Ok(())
    }

    fn get_provider_name(&self) -> String {
        "lm_studio".to_string()
    }
}

#[async_trait]
impl ChatCompletionProvider for LMStudioProvider {
    async fn create_streaming_chat_completion(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk, AIProviderError>> + Send>>, AIProviderError> {
        // Call the non-streaming completion
        let completion = self.create_chat_completion(request).await?;
        completion.choices
            .iter()
            .for_each(|choice| {
                // Log each choice for debugging
                log::debug!("Choice: {:?}", choice);
            });
        // Convert the completion into a single chunk (adapt as needed for your types)
        let chunk = ChatCompletionChunk {
            id: completion.id.clone(),
            choices: completion.choices.iter().map(|choice| ChatCompletionChunkChoice {
                delta: ChatMessageDelta {
                                role: None, // Roles typically come in the first chunk only
                                content: Some(choice.message.content.clone()),
                            },
                finish_reason: choice.finish_reason.clone(),
                index: choice.index,
            }).collect(),
            created: completion.created,
            // ...other fields as needed...
        };

        // Create a stream that yields this single chunk
        let stream = stream::once(async move { Ok(chunk) });

        Ok(Box::pin(stream) as Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk, AIProviderError>> + Send>>)
    }

    async fn create_chat_completion(
        &self, 
        request: &ChatCompletionRequest
    ) -> Result<ChatCompletionResponse, AIProviderError> {
        let url = format!("{}/chat/completions", self.base_url);
        
        // Serialize messages in OpenAI compatible format
        #[derive(Serialize, Debug)]
        struct LMStudioMessage {
            role: String,
            content: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
        }
        
        #[derive(Serialize, Debug)]
        struct LMStudioRequest {
            model: String,
            messages: Vec<LMStudioMessage>,
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_tokens: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            top_p: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            stop: Option<Vec<String>>,
        }
        
        let lm_messages: Vec<LMStudioMessage> = request.messages.iter()
            .map(|msg| {
                LMStudioMessage {
                    role: match msg.role {
                        MessageRole::System => "system".to_string(),
                        MessageRole::User => "user".to_string(),
                        MessageRole::Assistant => "assistant".to_string(),
                        MessageRole::Tool => "tool".to_string(),
                        MessageRole::Function => "function".to_string(),
                    },
                    content: msg.content.clone(),
                    name: msg.name.clone(),
                }
            })
            .collect();
            
        let lm_request = LMStudioRequest {
            model: request.model.clone(),
            messages: lm_messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            top_p: None,
            stop: None,
        };
        
        // Debug: Print the serialized JSON for inspection
        let json_string = serde_json::to_string_pretty(&lm_request)
            .map_err(|e| AIProviderError::APIError(format!("Failed to serialize request: {}", e)))?;
        
        log::debug!("LMStudio request JSON: {}", json_string);
        
        // Generate equivalent curl command for debugging - no auth header needed for LM Studio
        let curl_command = format!(
            "curl -X POST {} -H \"Content-Type: application/json\" --data '{}'",
            url,
            json_string.replace("'", "\\'") // Escape single quotes for shell safety
        );
        
        log::debug!("Equivalent curl command: \n{}", curl_command);
        
        let http_request = self.client.post(&url).json(&lm_request);
        let cloned_request = http_request.try_clone()
            .ok_or_else(|| AIProviderError::APIError("Failed to clone request".to_string()))?;
        // Add additional debugging for the raw request
        log::debug!("LMStudio request URL: {}", url);
        log::debug!("LMStudio request headers: {:?}", cloned_request.build().unwrap().headers());
        
        let response = http_request.send().await
            .map_err(|e| AIProviderError::APIError(format!("Network error: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
                
            return Err(AIProviderError::APIError(format!(
                "API returned error {}: {}", status, text
            )));
        }
        
        // Parse LM Studio response (OpenAI compatible format)
        #[derive(Deserialize)]
        struct LMStudioResponseMessage {
            role: String,
            content: String,
        }
        
        #[derive(Deserialize)]
        struct LMStudioResponseChoice {
            message: LMStudioResponseMessage,
            finish_reason: Option<String>,
            index: usize,
        }
        
        #[derive(Deserialize)]
        struct LMStudioUsage {
            prompt_tokens: u32,
            completion_tokens: u32,
            total_tokens: u32,
        }
        
        #[derive(Deserialize)]
        struct LMStudioResponse {
            id: String,
            object: String,
            created: u64,
            model: String,
            choices: Vec<LMStudioResponseChoice>,
            usage: Option<LMStudioUsage>,
        }
        
        let lm_response: LMStudioResponse = response.json().await
            .map_err(|e| AIProviderError::APIError(format!("Failed to parse response: {}", e)))?;
            
        // Convert to our generic format
        let choices = lm_response.choices.iter()
            .map(|choice| {
                let role = match choice.message.role.as_str() {
                    "system" => MessageRole::System,
                    "user" => MessageRole::User,
                    "assistant" => MessageRole::Assistant,
                    "tool" => MessageRole::Tool,
                    "function" => MessageRole::Function,
                    _ => MessageRole::Assistant, // Default
                };
                
                ChatCompletionChoice {
                    message: ChatMessage {
                        role,
                        content: choice.message.content.clone(),
                        name: None,
                    },
                    finish_reason: choice.finish_reason.clone(),
                    index: choice.index,
                }
            })
            .collect();
            
        let usage = lm_response.usage.map(|usage| {
            TokenUsage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            }
        });
            
        Ok(ChatCompletionResponse {
            id: lm_response.id,
            choices,
            created: lm_response.created,
            model: lm_response.model,
            usage,
        })
    }
}

#[async_trait]
impl EmbeddingProvider for LMStudioProvider {
    async fn create_embeddings(
        &self,
        embedding_request: EmbeddingRequest,
        // texts: &[String],
        // model: &str,
    ) -> Result<Vec<Embedding>, AIProviderError> {
        let url = format!("{}/embeddings", self.base_url);
        
        // LM Studio may not support embeddings, but we'll implement the API call
        // in case it does in the future or for compatible models
        let request_body = json!({
            "model": embedding_request.model,
            "input": embedding_request.input,
        });
        
        let http_request = self.client.post(&url).json(&request_body);
        let http_request = self.add_auth_header(http_request);
        
        let response = http_request.send().await
            .map_err(|e| AIProviderError::APIError(format!("Network error: {}", e)))?;
            
        if !response.status().is_success() {
            // Many local LLM servers don't support embeddings
            if response.status().as_u16() == 404 {
                return Err(AIProviderError::ModelNotAvailable(
                    format!("Embeddings not supported by this LM Studio instance")
                ));
            }
            
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
                
            return Err(AIProviderError::APIError(format!(
                "API returned error {}: {}", status, text
            )));
        }
        
        #[derive(Deserialize)]
        struct EmbeddingData {
            embedding: Vec<f32>,
            index: usize,
        }
        
        #[derive(Deserialize)]
        struct EmbeddingResponse {
            data: Vec<EmbeddingData>,
        }
        
        let embedding_response: EmbeddingResponse = response.json().await
            .map_err(|e| AIProviderError::APIError(format!("Failed to parse response: {}", e)))?;
        let embedding_model_name = PreferredEmbeddingModel::get_preferred_embedding_model(self);
        Ok(embedding_response.data.into_iter()
            .map(|e| Embedding {
                vector: e.embedding,
                index: e.index,
                model_name: Some(embedding_model_name.clone()),
            })
            .collect())
    }
}

impl PreferredEmbeddingModel for LMStudioProvider {
    fn get_preferred_embedding_model(&self) -> String {
        "text-embedding-nomic-embed-text-v1.5".to_string()
        // LM Studio doesn't have a preferred embedding model
        //unimplemented!("LM Studio does not yet support a preferred embedding model");
    }
    
}