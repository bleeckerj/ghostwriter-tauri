use async_trait::async_trait;
use reqwest::Client;
use std::path::Path;
use std::collections::HashMap;
use gray_matter::Pod;
use regex::Regex;
use super::*;
use log;
use std::any::Any;
use std::fs;
use std::io::Write;
use serde_yaml;

#[derive(Debug, Clone)]
pub struct UrlDocumentIngestor;

impl UrlDocumentIngestor {
    pub async fn ingest_url(&self, url: &str) -> Result<IngestedDocument, IngestError> {
        // 1. Construct the Jina Reader URL
        let encoded_url = reqwest::Url::parse(url)
        .map_err(|e| IngestError::Parse(format!("Invalid URL: {}", e)))?;
        let jina_api_url = format!("https://r.jina.ai/{}", encoded_url);
        
        log::info!("Fetching content from URL: {} via Jina Reader", url);
        
        let url_root = format!("{}{}",
        encoded_url.origin().unicode_serialization(),
        encoded_url.path());
        
        // 2. Create a client with a timeout
        let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        // .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .build()
        .map_err(|e| IngestError::Parse(format!("Failed to build HTTP client: {}", e)))?;
        
        // 3. Send GET request
        log::debug!("Sending request to: {}", jina_api_url);
        let response = client.get(&jina_api_url)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to connect to Jina Reader API: {}", e);
            IngestError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to connect to Jina Reader API: {}", e)
            ))
        })?;
        
        // 4. Check status before getting content
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            let error_msg = format!("Jina Reader API returned error {}: {}", status, error_text);
            log::error!("{}", error_msg);
            return Err(IngestError::Parse(error_msg));
        }
        
        // 5. Get the Markdown/text response
        let response_text = response.text().await
        .map_err(|e| IngestError::Parse(format!("Failed to read response body: {}", e)))?;
        
        // 6. Clean any HTML tags from the Markdown
        let re = Regex::new(r"<[^>]*>").unwrap(); // Regex to match HTML tags
        let cleaned_content = re.replace_all(&response_text, "").to_string();
        
        // 7. Extract title from first line if possible
        let mut title = url_root.to_string();
        let first_line = cleaned_content.lines().next();
        if let Some(line) = first_line {
            let trimmed = line.trim();
            // If line starts with # or ## it's likely a title
            if (trimmed.starts_with("# ") || trimmed.starts_with("## ")) && trimmed.len() > 3 {
                title = trimmed[2..].trim().to_string();
            }
        }
        
        // Generate current timestamp in RFC 3339 format
        let current_time: String = chrono::Local::now().to_rfc3339();
        
        // 8. Create document
        let mut frontmatter = HashMap::new();
        frontmatter.insert("url_root".to_string(), Pod::String(url_root.clone()));
        frontmatter.insert("url".to_string(), Pod::String(url.to_string()));
        frontmatter.insert("source".to_string(), Pod::String(jina_api_url.to_string()));
        frontmatter.insert("current_time".to_string(), Pod::String(current_time.clone()));
        
        
        Ok(IngestedDocument {
            title,  // Use extracted title or URL
            content: cleaned_content,
            metadata: DocumentMetadata {
                source_type: "URL".to_string(),
                source_path: url_root.to_string(),
                author: None,
                created_date: Some(current_time.clone()),  // Add current time as creation date
                modified_date: Some(current_time),         // Same for modification date    
                frontmatter,
            },
        })
    }
    
    /// Save the ingested document to a file
    /// 
    /// This method serializes the document content along with its metadata
    /// in a format that can be later loaded without having to fetch the URL again.
    pub fn save_to_file(&self, document: &IngestedDocument, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Create parent directories if they don't exist
        if let Some(parent) = Path::new(file_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Create a markdown file with front matter
        let mut front_matter = serde_json::Map::new();
        front_matter.insert("title".to_string(), serde_json::Value::String(document.title.clone()));
        front_matter.insert("url".to_string(), serde_json::Value::String(document.metadata.source_path.clone()));
        
        // Add any additional metadata
        if let Some(author) = &document.metadata.author {
            front_matter.insert("author".to_string(), serde_json::Value::String(author.clone()));
        }
        if let Some(created_date) = &document.metadata.created_date {
            front_matter.insert("created_date".to_string(), serde_json::Value::String(created_date.clone()));
        }
        if let Some(modified_date) = &document.metadata.modified_date {
            front_matter.insert("modified_date".to_string(), serde_json::Value::String(modified_date.clone()));
        }
        
        // Add the date this was saved
        front_matter.insert("saved_date".to_string(), 
        serde_json::Value::String(chrono::Local::now().to_rfc3339()));
        
        // Convert custom frontmatter from HashMap<String, Pod> to serde_json::Value
        for (key, value) in &document.metadata.frontmatter {
            match value {
                Pod::Boolean(b) => front_matter.insert(key.clone(), serde_json::Value::Bool(*b)),
                Pod::Integer(i) => front_matter.insert(key.clone(), serde_json::Value::Number(serde_json::Number::from(*i))),
                Pod::Float(f) => {
                    if let Some(num) = serde_json::Number::from_f64(*f) {
                        front_matter.insert(key.clone(), serde_json::Value::Number(num))
                    } else {
                        front_matter.insert(key.clone(), serde_json::Value::Null)
                    }
                },
                Pod::String(s) => front_matter.insert(key.clone(), serde_json::Value::String(s.clone())),
                Pod::Array(arr) => {
                    let json_arr = arr.iter().map(|p| Self::pod_to_json(p)).collect::<Vec<_>>();
                    front_matter.insert(key.clone(), serde_json::Value::Array(json_arr))
                },
                Pod::Hash(map) => {
                    let mut json_map = serde_json::Map::new();
                    for (k, v) in map {
                        json_map.insert(k.clone(), Self::pod_to_json(v));
                    }
                    front_matter.insert(key.clone(), serde_json::Value::Object(json_map))
                },
                Pod::Null => front_matter.insert(key.clone(), serde_json::Value::Null),
            };
        }
        
        // Create the content with YAML frontmatter
        let yaml_front_matter = serde_yaml::to_string(&front_matter)?;
        let content = format!("---\n{}---\n\n{}", yaml_front_matter, document.content);
        
        // Write to file
        let mut file = fs::File::create(file_path)?;
        file.write_all(content.as_bytes())?;
        
        log::info!("Saved URL document to {}", file_path);
        Ok(())
    }
    
    // Helper function to convert Pod to serde_json::Value
    fn pod_to_json(pod: &Pod) -> serde_json::Value {
        match pod {
            Pod::Boolean(b) => serde_json::Value::Bool(*b),
            Pod::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            Pod::Float(f) => {
                if let Some(num) = serde_json::Number::from_f64(*f) {
                    serde_json::Value::Number(num)
                } else {
                    serde_json::Value::Null
                }
            },
            Pod::String(s) => serde_json::Value::String(s.clone()),
            Pod::Array(arr) => {
                let json_arr = arr.iter().map(|p| Self::pod_to_json(p)).collect::<Vec<_>>();
                serde_json::Value::Array(json_arr)
            },
            Pod::Hash(map) => {
                let mut json_map = serde_json::Map::new();
                for (k, v) in map {
                    json_map.insert(k.clone(), Self::pod_to_json(v));
                }
                serde_json::Value::Object(json_map)
            },
            Pod::Null => serde_json::Value::Null,
        }
    }
    
}



#[async_trait]
impl DocumentIngestor for UrlDocumentIngestor {
    fn can_handle(&self, resource: &Resource) -> bool {
        match resource {
            Resource::FilePath(_) => false,
            Resource::Url(url) => url.starts_with("http"),
            Resource::Database(_) => false,
        }
    }
    
    async fn ingest(&self, resource: &Resource) -> Result<IngestedDocument, IngestError> {
        match resource {
            Resource::Url(url) => self.ingest_url(url).await,
            Resource::FilePath(path) => Err(IngestError::UnsupportedFormat(
                format!("UrlDocumentIngestor cannot process file paths: {}", path.display())
            )),
            Resource::Database(_) => Err(IngestError::UnsupportedFormat(
                "UrlDocumentIngestor cannot process database resources".to_string()
            )),
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self // This returns a reference to self as a type-erased &dyn Any
    }
}