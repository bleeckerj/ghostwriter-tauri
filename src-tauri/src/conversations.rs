#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]
pub struct Conversation {
    history: Vec<(String, String)>,
    max_chars: usize,
}

impl Conversation {
    pub fn new(max_chars: usize) -> Self {
        Conversation {
            history: Vec::new(),
            max_chars,
        }
    }

    pub fn add_exchange(&mut self, input: String, completion: String) {
        self.history.push((input, completion));
        self.truncate_if_needed();
    }

    pub fn get_context(&self) -> String {
        self.history
            .iter()
            .map(|(input, completion)| format!("Input: {}\nCompletion: {}\n", input, completion))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn get_history(&self) -> &Vec<(String, String)> {
        &self.history
    }

    fn truncate_if_needed(&mut self) {
        let mut total_chars = 0;
        let mut truncate_index = self.history.len();

        for (i, (input, completion)) in self.history.iter().rev().enumerate() {
            total_chars += input.len() + completion.len();
            if total_chars > self.max_chars {
                truncate_index = self.history.len() - i;
                break;
            }
        }

        if truncate_index < self.history.len() {
            self.history.drain(0..truncate_index);
        }
    }
}
