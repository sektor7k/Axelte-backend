pub mod scraper;
pub mod ai;
pub mod utils;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page {
    pub url: String,
    pub title: String,
    pub meta_description: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub headings: Vec<Heading>,
    pub paragraphs: Vec<String>,
    pub extract: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Heading {
    pub level: u8,
    pub text: String,
}

impl Page {
    pub fn new(url: String, title: String) -> Self {
        Self {
            url,
            title,
            meta_description: None,
            published_at: None,
            author: None,
            headings: Vec::new(),
            paragraphs: Vec::new(),
            extract: None,
        }
    }

    pub fn content_length(&self) -> usize {
        self.paragraphs.iter().map(|p| p.len()).sum()
    }

    pub fn truncate_content(&mut self, max_length: usize) {
        let mut current_length = 0;
        self.paragraphs.retain(|p| {
            if current_length + p.len() <= max_length {
                current_length += p.len();
                true
            } else {
                false
            }
        });
    }
} 