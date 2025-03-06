use crate::ingest::{
    DocumentIngestor, Resource, IngestedDocument, DocumentMetadata, IngestError,
    url_ingestor::UrlDocumentIngestor, DatabaseQuery,
};
// Import with alias to avoid namespace collision with mongodb
use crate::ingest::QueryParams as DocumentQueryParams;

use async_trait::async_trait;
use bson::{doc, Document, Bson};
use futures::stream::StreamExt;
use mongodb::{
    Client as MongoClient, 
    options::{ClientOptions, FindOptions},
    Collection, Database,
};
use gray_matter::Pod;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use url::Url;
use std::sync::Arc;
use log;

// #[derive(Debug, Default)]
// pub struct QueryParams {
//     pub message_id: Option<String>,
//     pub timestamp_from: Option<i64>,
//     pub timestamp_to: Option<i64>,
//     pub channel_id: Option<String>,
//     pub author_id: Option<String>,
//     pub keyword: Option<String>,
//     pub limit: Option<i64>,
// }

/// A structure representing MongoDB connection configuration
#[derive(Debug, Clone)]
pub struct MongoConfig {
    pub connection_string: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub edgar_bob_db_name: String,
    pub augie_bot_db_name: String,
}

impl Default for MongoConfig {
    fn default() -> Self {
        Self {
            // Default to local connection with no auth
            connection_string: "mongodb://localhost:27017".to_string(),
            username: None,
            password: None,
            edgar_bob_db_name: "Edgar_Bob".to_string(),
            augie_bot_db_name: "AugieBot".to_string(),
        }
    }
}

impl MongoConfig {
    /// Create a new config for MongoDB Atlas
    pub fn new_atlas(username: &str, password: &str, cluster: &str) -> Self {
        Self {
            connection_string: format!("mongodb+srv://{}:{}@{}", username, password, cluster),
            username: Some(username.to_string()),
            password: Some(password.to_string()),
            edgar_bob_db_name: "Edgar_Bob".to_string(),
            augie_bot_db_name: "AugieBot".to_string(),
        }
    }
    
    /// Get a full connection string with auth if provided
    pub fn get_connection_string(&self) -> String {
        // If the connection string already contains auth info or no auth is provided, use it as is
        if self.connection_string.contains('@') || 
        (self.username.is_none() && self.password.is_none()) {
            return self.connection_string.clone();
        }
        
        // Otherwise build a connection string with auth
        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            // Extract host part from connection string
            if let Some(host_part) = self.connection_string.strip_prefix("mongodb://") {
                return format!("mongodb://{}:{}@{}", user, pass, host_part);
            }
            if let Some(host_part) = self.connection_string.strip_prefix("mongodb+srv://") {
                return format!("mongodb+srv://{}:{}@{}", user, pass, host_part);
            }
        }
        
        // Return original if we couldn't modify it
        self.connection_string.clone()
    }
}

#[derive(Debug, Clone)]
pub struct MongoDocumentIngestor {
    config: MongoConfig,
    url_ingestor: UrlDocumentIngestor,
}

impl MongoDocumentIngestor {
    pub fn new(config: MongoConfig) -> Self {
        MongoDocumentIngestor {
            config,
            url_ingestor: UrlDocumentIngestor,
        }
    }
    
    /// Extract URLs from text content
    fn extract_urls(&self, text: &str) -> Vec<String> {
        // Simple URL regex - this can be improved for more accuracy
        let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
        url_regex.find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
    }
    
    /// Connect to MongoDB and fetch digested messages with their original content
    async fn fetch_digested_messages(&self, message_id: Option<&str>) -> Result<Vec<Document>, IngestError> {
        log::info!("Connecting to MongoDB with connection string: {}", self.config.get_connection_string());
        
        // Connect to MongoDB
        let client_options = ClientOptions::parse(&self.config.get_connection_string())
        .await
        .map_err(|e| IngestError::Parse(format!("Failed to parse MongoDB connection string: {}", e)))?;
        
        let client = MongoClient::with_options(client_options)
        .map_err(|e| IngestError::Parse(format!("Failed to create MongoDB client: {}", e)))?;
        
        // Get handles to the databases
        let augie_bot_db = client.database(&self.config.augie_bot_db_name);
        let edgar_bob_db = client.database(&self.config.edgar_bob_db_name);
        
        // Get collection handles
        let digest_messages: Collection<Document> = augie_bot_db.collection("digest_messages");
        let messages: Collection<Document> = edgar_bob_db.collection("messages");
        
        // Build the query
        let mut query = doc! {};
        if let Some(id) = message_id {
            query = doc! { "originalMessageId": id };
        }
        
        // Find digested messages
        let mut results = Vec::new();
        let mut cursor = digest_messages.find(query).await
        .map_err(|e| IngestError::Parse(format!("Failed to query digest_messages: {}", e)))?;
        
        while let Some(digest_result) = cursor.next().await {
            let digest = digest_result
            .map_err(|e| IngestError::Parse(format!("Error iterating digest messages: {}", e)))?;
            
            // Extract the original message ID
            let original_id = digest.get("originalMessageId")
            .and_then(|id| id.as_str())
            .ok_or_else(|| IngestError::Parse("Missing originalMessageId in digest message".to_string()))?;
            
            // Find the corresponding message in Edgar_Bob
            let message_query = doc! { "id": original_id };
            let message_result = messages.find_one(message_query).await
            .map_err(|e| IngestError::Parse(format!("Failed to query messages: {}", e)))?;
            
            if let Some(message) = message_result {
                // Combine the digest and message
                let mut combined = Document::new();
                combined.insert("digest", Bson::Document(digest));
                combined.insert("message", Bson::Document(message));
                results.push(combined);
            }
        }
        
        log::info!("Found {} digested messages with their original content", results.len());
        Ok(results)
    }
    
    // Process a specific message ID
    async fn process_message(&self, message_id: &str) -> Result<IngestedDocument, IngestError> {
        let combined_documents = self.fetch_digested_messages(Some(message_id)).await?;
        if combined_documents.is_empty() {
            return Err(IngestError::Parse(format!("No document found for message ID: {}", message_id)));
        }
        
        self.process_documents(combined_documents, Some(message_id)).await
    }
    
    // Process all messages or a filtered subset
    async fn process_documents(&self, documents: Vec<Document>, message_id: Option<&str>) -> Result<IngestedDocument, IngestError> {
        let mut all_content = String::new();
        let mut all_urls = Vec::new();
        let mut title = message_id
        .map(|id| format!("Digested message {}", id))
        .unwrap_or_else(|| "MongoDB Digested Messages".to_string());
        
        for doc in &documents {
            // Extract content from the message
            if let Some(message) = doc.get("message").and_then(|m| m.as_document()) {
                if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                    all_content.push_str(content);
                    all_content.push_str("\n\n");
                    
                    // Extract URLs from content
                    all_urls.extend(self.extract_urls(content));
                    log::debug!("Found {:?} URLs in message content", all_urls);
                }
            }
        }
        
        // Ingest content from all found URLs
        let mut url_contents = Vec::new();
        for url in all_urls {
            log::info!("Ingesting content from URL: {}", url);
            match self.url_ingestor.ingest_url(&url).await {
                Ok(doc) => {
                    url_contents.push(format!("This is the content of {}:\n{}", url, doc.content));
                    println!("Ingested URL: {}", url);
                    println!("Contents {} {}", doc.content.len(), doc.content);
                }
                Err(e) => {
                    log::warn!("Failed to ingest URL {}: {}", url, e);
                }
            }
        }
        
        // Combine original messages with URL contents
        if !url_contents.is_empty() {
            all_content.push_str("\n\n");
            all_content.push_str(&url_contents.join("\n\n"));
        }
        
        // Create frontmatter for metadata
        let mut frontmatter = HashMap::new();
        frontmatter.insert("source_type".to_string(), Pod::String("mongodb".to_string()));
        
        if let Some(id) = message_id {
            frontmatter.insert("message_id".to_string(), Pod::String(id.to_string()));
        }
        
        frontmatter.insert("documents_count".to_string(), Pod::Integer(documents.len() as i64));
        
        // Return the ingested document
        Ok(IngestedDocument {
            title,
            content: all_content,
            metadata: DocumentMetadata {
                source_type: "MongoDB".to_string(),
                source_path: format!("{}/{}", self.config.augie_bot_db_name, self.config.edgar_bob_db_name),
                author: None,
                created_date: None,
                modified_date: None,
                frontmatter,
            },
        })
    }
    
    async fn fetch_digested_messages_with_params(&self, params: &DocumentQueryParams) -> Result<Vec<Document>, IngestError> {
        log::info!("Connecting to MongoDB");
        
        // Get connection string with auth if needed
        let connection_string = self.config.get_connection_string();
        
        // Connect to MongoDB
        let client_options = ClientOptions::parse(&connection_string).await
            .map_err(|e| IngestError::Parse(format!("Failed to parse MongoDB connection string: {}", e)))?;
        
        let client = MongoClient::with_options(client_options)
            .map_err(|e| IngestError::Parse(format!("Failed to create MongoDB client: {}", e)))?;
        
        // Get database handles
        let augie_bot_db = client.database(&self.config.augie_bot_db_name);
        let edgar_bob_db = client.database(&self.config.edgar_bob_db_name);
        
        // Get collection handles
        let digest_messages: Collection<Document> = augie_bot_db.collection("digest_messages");
        let messages: Collection<Document> = edgar_bob_db.collection("messages");
        
        // Build query filter for digest_messages
        let mut digest_filter = doc! {};
        if let Some(message_id) = &params.message_id {
            digest_filter.insert("originalMessageId", message_id);
        }
        
        // Query digest_messages to get the list of originalMessageId
        let mut digest_cursor = digest_messages.find(digest_filter).await
            .map_err(|e| IngestError::Parse(format!("Failed to query digest_messages: {}", e)))?;
        
        let mut original_message_ids = Vec::new();
        while let Some(digest_result) = digest_cursor.next().await {
            let digest = digest_result
                .map_err(|e| IngestError::Parse(format!("Error iterating digest messages: {}", e)))?;
            
            if let Some(original_message_id) = digest.get("originalMessageId").and_then(|id| id.as_str()) {
                original_message_ids.push(original_message_id.to_string());
            }
        }
        
        // Build query filter for messages based on originalMessageId
        let mut message_filter = doc! { "id": { "$in": &original_message_ids } };
        
        // Add additional filters based on parameters
        if let Some(channel_id) = &params.channel_id {
            message_filter.insert("channelId", channel_id);
        }
        
        if let Some(author_id) = &params.author_id {
            message_filter.insert("authorId", author_id);
        }
        
        if let Some(keyword) = &params.keyword {
            // Text search (case insensitive)
            message_filter.insert("content", doc! { 
                "$regex": keyword, 
                "$options": "i" 
            });
        }
        
        // Time range query
        if params.timestamp_from.is_some() || params.timestamp_to.is_some() {
            let mut timestamp_filter = doc! {};
            
            if let Some(from) = params.timestamp_from {
                timestamp_filter.insert("$gte", from);
            }
            
            if let Some(to) = params.timestamp_to {
                timestamp_filter.insert("$lte", to);
            }
            
            if !timestamp_filter.is_empty() {
                message_filter.insert("createdTimestamp", timestamp_filter);
            }
        }
        
        // Setup find options (limit, sort, etc)
        let mut find_options = FindOptions::default();
        if let Some(limit) = params.limit {
            find_options.limit = Some(limit);
        }
        find_options.sort = Some(doc! { "createdTimestamp": -1 }); // Most recent first
        
        // Query messages collection
        let mut message_cursor = messages.find(message_filter).with_options(find_options).await
            .map_err(|e| IngestError::Parse(format!("Failed to query messages: {}", e)))?;
        
        // Initialize results vector
        let mut results = Vec::new();
        
        // Process results
        while let Some(message_result) = message_cursor.next().await {
            let message = message_result
                .map_err(|e| IngestError::Parse(format!("Error iterating messages: {}", e)))?;
            
            // Find the corresponding digest message
            let original_message_id = message.get("id")
                .and_then(|id| id.as_str())
                .ok_or_else(|| IngestError::Parse("Missing id in message".to_string()))?;
            
            let digest_query = doc! { "originalMessageId": original_message_id };
            let digest_result = digest_messages.find_one(digest_query).await
                .map_err(|e| IngestError::Parse(format!("Failed to query digest_messages: {}", e)))?;
            
            if let Some(digest) = digest_result {
                // Combine the digest and message
                let mut combined = Document::new();
                combined.insert("digest", Bson::Document(digest));
                combined.insert("message", Bson::Document(message));
                results.push(combined);
            }
        }
        
        log::info!("Found {} digested messages with their original content", results.len());
        Ok(results)
    }

    async fn fetch_messages_with_params(&self, params: &DocumentQueryParams) -> Result<Vec<Document>, IngestError> {
        log::info!("Connecting to MongoDB");
        
        // Get connection string with auth if needed
        let connection_string = self.config.get_connection_string();
        
        // Connect to MongoDB
        let client_options = ClientOptions::parse(&connection_string).await
            .map_err(|e| IngestError::Parse(format!("Failed to parse MongoDB connection string: {}", e)))?;
        
        let client = MongoClient::with_options(client_options)
            .map_err(|e| IngestError::Parse(format!("Failed to create MongoDB client: {}", e)))?;
        
        // Get database handle
        let edgar_bob_db = client.database(&self.config.edgar_bob_db_name);
        
        // Get collection handle
        let messages: Collection<Document> = edgar_bob_db.collection("messages");
        
        // Build query filter for messages
        let mut message_filter = doc! {};
        
        // Add additional filters based on parameters
        if let Some(message_id) = &params.message_id {
            message_filter.insert("id", message_id);
        }
        
        if let Some(channel_id) = &params.channel_id {
            message_filter.insert("channelId", channel_id);
        }
        
        if let Some(author_id) = &params.author_id {
            message_filter.insert("authorId", author_id);
        }
        
        if let Some(keyword) = &params.keyword {
            // Text search (case insensitive)
            message_filter.insert("content", doc! { 
                "$regex": keyword, 
                "$options": "i" 
            });
        }
        
        // Time range query
        if params.timestamp_from.is_some() || params.timestamp_to.is_some() {
            let mut timestamp_filter = doc! {};
            
            if let Some(from) = params.timestamp_from {
                timestamp_filter.insert("$gte", from);
            }
            
            if let Some(to) = params.timestamp_to {
                timestamp_filter.insert("$lte", to);
            }
            
            if !timestamp_filter.is_empty() {
                message_filter.insert("createdTimestamp", timestamp_filter);
            }
        }
        
        // Setup find options (limit, sort, etc)
        let mut find_options = FindOptions::default();
        if let Some(limit) = params.limit {
            find_options.limit = Some(limit);
        }
        find_options.sort = Some(doc! { "createdTimestamp": -1 }); // Most recent first
        
        // Query messages collection
        let mut message_cursor = messages.find(message_filter).with_options(find_options).await
            .map_err(|e| IngestError::Parse(format!("Failed to query messages: {}", e)))?;
        
        // Initialize results vector
        let mut results = Vec::new();
        
        // Process results
        while let Some(message_result) = message_cursor.next().await {
            let message = message_result
                .map_err(|e| IngestError::Parse(format!("Error iterating messages: {}", e)))?;
            
            results.push(message);
        }
        
        log::info!("Found {} messages", results.len());
        Ok(results)
    }

    pub async fn ingest_with_digest_option(&self, resource: &Resource, digest_only: bool) -> Result<IngestedDocument, IngestError> {
        match resource {
            Resource::Database(query) => {
                // Handle structured database query
                let documents = if digest_only {
                    self.fetch_digested_messages_with_params(&query.query_params).await?
                } else {
                    self.fetch_messages_with_params(&query.query_params).await?
                };
                self.process_documents(documents, query.query_params.message_id.as_deref()).await
            },
            
            Resource::Url(url) if url.starts_with("mongodb://") => {
                // Simple connection string with no parameters
                // Default to retrieving everything
                let documents = self.fetch_digested_messages_with_params(&DocumentQueryParams::default()).await?;
                self.process_documents(documents, None).await
            },
            
            Resource::FilePath(path) => {
                let path_str = path.to_string_lossy();
                if path_str.starts_with("mongodb:") {
                    // Format is mongodb:message_id
                    let message_id = path_str.strip_prefix("mongodb:").unwrap();
                    
                    let mut params = DocumentQueryParams::default();
                    params.message_id = Some(message_id.to_string());
                    
                    let documents = self.fetch_digested_messages_with_params(&params).await?;
                    self.process_documents(documents, Some(message_id)).await
                } else {
                    Err(IngestError::UnsupportedFormat(
                        format!("MongoDocumentIngestor cannot process file path: {}", path.display())
                    ))
                }
            },
            
            _ => Err(IngestError::UnsupportedFormat(
                "MongoDocumentIngestor can only process MongoDB resources".to_string()
            )),
        }
    }
}

#[async_trait]
impl DocumentIngestor for MongoDocumentIngestor {
    fn can_handle(&self, resource: &Resource) -> bool {
        match resource {
            Resource::Url(url) => url.starts_with("mongodb://"),
            Resource::FilePath(path) => path.to_string_lossy().starts_with("mongodb:"),
            Resource::Database(query) => query.connection_string.starts_with("mongodb://"),
        }
    }
    
    async fn ingest(&self, resource: &Resource) -> Result<IngestedDocument, IngestError> {
        match resource {
            Resource::Database(query) => {
                // Handle structured database query
                let documents = self.fetch_digested_messages_with_params(&query.query_params).await?;
                self.process_documents(documents, query.query_params.message_id.as_deref()).await
            },
            
            Resource::Url(url) if url.starts_with("mongodb://") => {
                // Simple connection string with no parameters
                // Default to retrieving everything
                let documents = self.fetch_digested_messages_with_params(&DocumentQueryParams::default()).await?;
                self.process_documents(documents, None).await
            },
            
            Resource::FilePath(path) => {
                let path_str = path.to_string_lossy();
                if path_str.starts_with("mongodb:") {
                    // Format is mongodb:message_id
                    let message_id = path_str.strip_prefix("mongodb:").unwrap();
                    
                    let mut params = DocumentQueryParams::default();
                    params.message_id = Some(message_id.to_string());
                    
                    let documents = self.fetch_digested_messages_with_params(&params).await?;
                    self.process_documents(documents, Some(message_id)).await
                } else {
                    Err(IngestError::UnsupportedFormat(
                        format!("MongoDocumentIngestor cannot process file path: {}", path.display())
                    ))
                }
            },
            
            _ => Err(IngestError::UnsupportedFormat(
                "MongoDocumentIngestor can only process MongoDB resources".to_string()
            )),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mongodb_ingestor() {
        // Read password from environment variable to avoid hardcoding
        let password = std::env::var("MONGODB_TEST_PASSWORD")
            .expect("Set MONGODB_TEST_PASSWORD env var to run this test");
        
        // Create MongoDB configuration with test credentials
        let mongo_config = MongoConfig {
            connection_string: "mongodb+srv://edgarcluster-01.gph8w.mongodb.net".to_string(),
            username: Some("jayedgar".to_string()),
            password: Some(password),
            edgar_bob_db_name: "Edgar_Bob".to_string(),
            augie_bot_db_name: "AugieBot".to_string(),
        };
        
        let query_params = DocumentQueryParams {
            message_id: None, //Some("988132972572065902".to_string()), // Use a known message ID
            channel_id: Some("963630130805239858".to_string()), // Example channel ID
            author_id: None, //Some("758825074031067138".to_string()), // Example author ID
            timestamp_from: None, //Some(1739577600000), // Example timestamp range
            timestamp_to: None, //Some(1651400000000),
            keyword: None,
            limit: Some(5), // Limit the number of results
        };

        // Create the ingestor
        let ingestor = MongoDocumentIngestor::new(mongo_config);
        
        // Test a specific message ID
        let resource = Resource::Database(DatabaseQuery {
            connection_string: "mongodb+srv://edgarcluster-01.gph8w.mongodb.net".to_string(),
            database_name: "AugieBot".to_string(),
            collection_name: "digest_messages".to_string(),
            query_params,
        });
        
        // Run the test with digest_only = true
        match ingestor.ingest_with_digest_option(&resource, true).await {
            Ok(doc) => {
                println!("✅ Successfully ingested document with digest_only = true!");
                println!("Title: {}", doc.title);
                println!("Content length: {} bytes", doc.content.len());
                println!("First 1024 chars: {}", &doc.content[..1024.min(doc.content.len())]);
                assert!(!doc.content.is_empty(), "Document content should not be empty");
            },
            Err(e) => {
                println!("❌ Error ingesting with digest_only = true: {}", e);
                panic!("Test failed: {}", e); // Fail the test if there's an error
            }
        }

        // Run the test with digest_only = false
        match ingestor.ingest_with_digest_option(&resource, false).await {
            Ok(doc) => {
                println!("✅ Successfully ingested document with digest_only = false!");
                println!("Title: {}", doc.title);
                println!("Content length: {} bytes", doc.content.len());
                println!("First 1024 chars: {}", &doc.content[..1024.min(doc.content.len())]);
                assert!(!doc.content.is_empty(), "Document content should not be empty");
            },
            Err(e) => {
                println!("❌ Error ingesting with digest_only = false: {}", e);
                panic!("Test failed: {}", e); // Fail the test if there's an error
            }
        }
    }
}
