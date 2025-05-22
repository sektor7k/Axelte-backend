use crate::Page;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

pub struct AIAnalyzer {
    client: reqwest::Client,
    api_key: String,
}

impl AIAnalyzer {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    pub async fn analyze_pages(&self, pages: &[Page]) -> Result<String, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let payload = json!({
            "model": "gpt-4-turbo-preview",
            "messages": [
                {
                    "role": "system",
                    "content": "You are an expert blockchain and crypto project analyst. You evaluate whitepapers, tokenomics, roadmap, team ve kullanım senaryoları konusunda derin bilgi sahibisiniz. Cevaplarınızı net başlıklar (Overview, Use Case, Tokenomics, Roadmap, Team, Risks, Summary) halinde, kısa ve öz ama kapsamlı olacak şekilde düzenleyin."
                },
                {
                    "role": "user",
                    "content": format!(
                        "Aşağıdaki web sitesi içeriklerini analiz et ve blockchain/crypto projesi olarak değerlendir:\n\n{}",
                        pages.iter().map(|p| format!(
                            "URL: {}\nTitle: {}\nMeta: {}\nAuthor: {}\nDate: {}\nHeadings: {}\nContent: {}\nExtract: {}\n\n",
                            p.url,
                            p.title,
                            p.meta_description.as_deref().unwrap_or("N/A"),
                            p.author.as_deref().unwrap_or("N/A"),
                            p.published_at.map(|d| d.to_rfc3339()).unwrap_or_else(|| "N/A".to_string()),
                            p.headings.iter().map(|h| format!("{}: {}", h.level, h.text)).collect::<Vec<_>>().join(", "),
                            p.paragraphs.join("\n"),
                            p.extract.as_deref().unwrap_or("N/A")
                        )).collect::<String>()
                    )
                }
            ],
            "temperature": 0.7,
            "max_tokens": 4000
        });

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        let result = response.json::<serde_json::Value>().await?;
        Ok(result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Analiz yapılamadı")
            .to_string())
    }
}