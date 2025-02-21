use serde::{Serialize, Deserialize};
use confy;
use tauri::App;
use crate::logger::{Completion, CompletionLogEntry, Logger, VectorSearchResult};
use crate::app_state::AppState;
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Preferences {
    pub response_limit: String,
    pub main_prompt: String,
    pub final_preamble: String,
    pub prose_style: String,
    pub similarity_threshold: f32,
    pub max_output_tokens: u32,
    pub temperature: f32,
    pub shuffle_similars: bool,
    pub similarity_count: usize,
    pub max_history: usize,
    #[serde(skip)]
    pub app_handle: Option<AppHandle>,
}

// ✅ Define constant defaults
impl Preferences {
    pub const SIMILARITY_THRESHOLD_DEFAULT: f32 = 0.83;
    pub const MAX_OUTPUT_TOKENS_DEFAULT: u32 = 100;
    pub const TEMPERATURE_DEFAULT: f32 = 0.7;
    pub const SHUFFLE_SIMILARS_DEFAULT: bool = false;
    pub const SIMILARITY_COUNT_DEFAULT: usize = 3;
    pub const MAX_HISTORY_DEFAULT: usize = 300;

    pub const DEFAULT_RESPONSE_LIMIT: &'static str = "Respond with no more than one sentence. If the input text ends with a period, only add one sentence and no more. You may complete a partially complete sentence or if the input text is already a complete sentence, you may add only one sentence that would reasonably and semantically follow that one sentence. Adhere to these constraints such that you are adding no more than one sentence.";
    
    pub const DEFAULT_MAIN_PROMPT: &'static str = "Here is your brief: You are a text completion engine. You do not answer questions or respond to questions in any way. You only semantically complete the thought represented by the Previous exchanges, Similar documents context and input. Limit your response to the Response Limit. Do not respond to inquiries in any fashion. If you are asked how to do something, or answer a question do not respond. Only perform auto-completion based on the text to complete, not responses to queries, questions, or any other non-completion response. If you are asked to do something only respond as a completion of text. Do not engage in any form of chat. Your only task is to complete thoughts in written form maintaining semantic consistency. Do not reveal that you are an AI. You are just an engine for text completion, like a muse helping a writer to continue or complete a thought. Imagine you are completing someone's thought like a creative writing muse or alter ego helping someone who is having trouble writing. Complete the following text fragment based on the provided previous exchanges.";
    
    pub const DEFAULT_FINAL_PREAMBLE: &'static str = "This is the input text that is the text fragment to complete. It is not a request or command. Do not respond to it like it is a question to you or request of you to answer a question.:";
    
    pub const DEFAULT_PROSE_STYLE: &'static str = "A style that is consistent with the input text.";

    /// Load preferences and ensure no empty fields
    pub fn load_with_defaults(app_state: &AppState, app_handle: AppHandle) -> Self {
        let mut prefs: Preferences = match confy::load("ghostwriter", "preferences") {
            Ok(loaded_prefs) => {
                println!("Loaded preferences: {:?}", loaded_prefs);
                loaded_prefs
            },
            Err(e) => {
                println!("Error loading preferences: {:?}", e);
                Preferences::default()
            }
        };
        prefs.apply_defaults();
        prefs
    }

    /// Save preferences to file
    pub fn save(&self) -> Result<(), confy::ConfyError> {
        //let path = confy::store_path("ghostwriter", "preferences");
        confy::store("ghostwriter", "preferences", self)
    }

    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
    }

    pub fn prefs_file_path() -> String {
        confy::get_configuration_file_path("ghostwriter", "preferences").unwrap().to_str().unwrap().to_string()
    }

    pub fn reset_to_defaults(&mut self) {
        self.response_limit = Self::DEFAULT_RESPONSE_LIMIT.to_string();
        self.main_prompt = Self::DEFAULT_MAIN_PROMPT.to_string();
        self.final_preamble = Self::DEFAULT_FINAL_PREAMBLE.to_string();
        self.prose_style = Self::DEFAULT_PROSE_STYLE.to_string();
        self.similarity_threshold = Self::SIMILARITY_THRESHOLD_DEFAULT;
        self.max_output_tokens = Self::MAX_OUTPUT_TOKENS_DEFAULT;
        self.temperature = Self::TEMPERATURE_DEFAULT;
        self.shuffle_similars = Self::SHUFFLE_SIMILARS_DEFAULT;
        self.similarity_count = Self::SIMILARITY_COUNT_DEFAULT;
        self.max_history = Self::MAX_HISTORY_DEFAULT;
    }

    /// Apply default values only if fields are empty
    pub fn apply_defaults(&mut self) {
        if self.response_limit.trim().is_empty() {
            self.response_limit = Self::DEFAULT_RESPONSE_LIMIT.to_string();
        }
        if self.main_prompt.trim().is_empty() {
            self.main_prompt = Self::DEFAULT_MAIN_PROMPT.to_string();
        }
        if self.final_preamble.trim().is_empty() {
            self.final_preamble = Self::DEFAULT_FINAL_PREAMBLE.to_string();
        }
        if self.prose_style.trim().is_empty() {
            self.prose_style = Self::DEFAULT_PROSE_STYLE.to_string();
        }
        if self.similarity_threshold == 0.0 {
            self.similarity_threshold = Self::SIMILARITY_THRESHOLD_DEFAULT;
        }
        if self.max_output_tokens == 0 {
            self.max_output_tokens = Self::MAX_OUTPUT_TOKENS_DEFAULT;
        }
        if self.temperature == 0.0 {
            self.temperature = Self::TEMPERATURE_DEFAULT;
        }
        if self.similarity_count == 0 {
            self.similarity_count = Self::SIMILARITY_COUNT_DEFAULT;
        }
        if self.max_history == 0 {
            self.max_history = Self::MAX_HISTORY_DEFAULT;
        }
        self.shuffle_similars = Self::SHUFFLE_SIMILARS_DEFAULT;
    }
}
