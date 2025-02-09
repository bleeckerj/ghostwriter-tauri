#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
use crate::conversations::Conversation;
use crate::document_store::DocumentStore;
use crate::embeddings::EmbeddingGenerator;
//use crate::logger::Logger;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub doc_store: Arc<Mutex<DocumentStore>>,
    pub embedding_generator: Arc<EmbeddingGenerator>,
    pub conversation: Mutex<Conversation>,
    pub buffer: Mutex<String>,
    //pub logger: Mutex<Logger>,
}

impl AppState {
    pub fn new(
        doc_store: DocumentStore,
        embedding_generator: EmbeddingGenerator,
        log_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize logger
        //let logger = Logger::new(log_path)?;

        Ok(Self {
            //logger: Mutex::new(logger),
            doc_store: Arc::new(Mutex::new(doc_store)),
            embedding_generator: Arc::new(embedding_generator),
            conversation: Mutex::new(Conversation::new(16000)),
            buffer: Mutex::new(String::new()),
        })
    }
}
