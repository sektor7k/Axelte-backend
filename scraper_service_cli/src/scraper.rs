use crate::{Page, Heading};
use futures::stream::{self, StreamExt};
use reqwest::Client;
use scraper::{Html, Selector};
use std::{collections::HashSet, time::Duration};
use tokio::sync::Mutex;
use url::Url;
use backoff::{ExponentialBackoff, future::retry};
use chrono::Utc;

pub struct Scraper {
    client: Client,
    base_url: Url,
    visited: Mutex<HashSet<String>>,
    pages: Mutex<Vec<Page>>,
    max_pages: usize,
    concurrent_requests: usize,
    max_content_length: usize,
}

impl Scraper {
    pub fn new(start_url: &str, max_pages: usize, concurrent_requests: usize, max_content_length: usize) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?,
            base_url: Url::parse(start_url)?,
            visited: Mutex::new(HashSet::new()),
            pages: Mutex::new(Vec::new()),
            max_pages,
            concurrent_requests,
            max_content_length,
        })
    }

    pub async fn scrape(&self, start_url: &str) -> Result<Vec<Page>, Box<dyn std::error::Error>> {
        let mut urls_to_visit = vec![start_url.to_string()];
        
        while !urls_to_visit.is_empty() {
            let batch: Vec<_> = urls_to_visit.drain(..self.concurrent_requests.min(urls_to_visit.len())).collect();
            
            let mut new_urls = stream::iter(batch)
                .map(|url| self.visit_with_retry(url))
                .buffer_unordered(self.concurrent_requests)
                .collect::<Vec<_>>()
                .await;

            urls_to_visit.append(&mut new_urls.into_iter().flatten().collect());
            
            if self.pages.lock().await.len() >= self.max_pages {
                break;
            }
        }

        Ok(self.pages.lock().await.clone())
    }

    async fn visit_with_retry(&self, url: String) -> Vec<String> {
        let backoff = ExponentialBackoff::default();
        
        match retry(backoff, || async {
            match self.visit(&url).await {
                Ok(urls) => Ok(urls),
                Err(e) => {
                    eprintln!("❌ {} için hata: {}", url, e);
                    Err(backoff::Error::transient(e))
                }
            }
        }).await {
            Ok(urls) => urls,
            Err(e) => {
                eprintln!("❌ {} için maksimum deneme sayısına ulaşıldı: {}", url, e);
                Vec::new()
            }
        }
    }

    async fn visit(&self, url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if !self.visited.lock().await.insert(url.to_string()) {
            return Ok(Vec::new());
        }

        println!("🔍 Ziyaret ediliyor: {}", url);
        
        // Rate limiting için kısa bir bekleme
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        let res = self.client.get(url).send().await?;

        if !res.status().is_success() {
            eprintln!("❌ {} HTTP error: {}", url, res.status());
            return Ok(Vec::new());
        }

        if let Some(server) = res.headers().get("server") {
            let srv = server.to_str()?.to_lowercase();
            if srv.contains("cloudflare") {
                eprintln!("⚠️ {} Cloudflare korumalı, scraping durduruluyor.", url);
                return Ok(Vec::new());
            }
        }

        let body = res.text().await?;
        if body.contains("Attention Required!") || body.contains("Checking your browser") {
            eprintln!("⚠️ {} bot koruma sayfası döndürüyor.", url);
            return Ok(Vec::new());
        }

        let doc = Html::parse_document(&body);
        self.pages.lock().await.push(self.format_page(url, &doc));

        let mut new_urls = Vec::new();
        let link_selector = Selector::parse("a").unwrap();
        
        for el in doc.select(&link_selector) {
            if let Some(href) = el.value().attr("href") {
                if let Ok(link) = self.base_url.join(href) {
                    if link.domain() == self.base_url.domain() {
                        new_urls.push(link.into_string());
                    }
                }
            }
        }

        Ok(new_urls)
    }

    fn format_page(&self, url: &str, doc: &Html) -> Page {
        let title_selector = Selector::parse("title").unwrap();
        let meta_desc_selector = Selector::parse("meta[name='description']").unwrap();
        let heading_selector = Selector::parse("h1, h2, h3, h4, h5, h6").unwrap();
        let paragraph_selector = Selector::parse("p").unwrap();
        let time_selector = Selector::parse("time").unwrap();
        let author_selector = Selector::parse("meta[name='author']").unwrap();

        let title = doc
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_else(|| "Başlık bulunamadı".to_string());

        let mut page = Page::new(url.to_string(), title.trim().to_string());

        // Meta description
        if let Some(meta) = doc.select(&meta_desc_selector).next() {
            if let Some(content) = meta.value().attr("content") {
                page.meta_description = Some(content.to_string());
            }
        }

        // Author
        if let Some(author) = doc.select(&author_selector).next() {
            if let Some(content) = author.value().attr("content") {
                page.author = Some(content.to_string());
            }
        }

        // Published date
        if let Some(time) = doc.select(&time_selector).next() {
            if let Some(datetime) = time.value().attr("datetime") {
                if let Ok(date) = chrono::DateTime::parse_from_rfc3339(datetime) {
                    page.published_at = Some(date.with_timezone(&Utc));
                }
            }
        }

        // Headings
        for el in doc.select(&heading_selector) {
            let level = el.value().name().chars().nth(1).unwrap_or('1').to_digit(10).unwrap_or(1) as u8;
            let text = el.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                page.headings.push(Heading { level, text });
            }
        }

        // Paragraphs
        for el in doc.select(&paragraph_selector) {
            let text = el.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                page.paragraphs.push(text);
            }
        }

        // Truncate content if needed
        if page.content_length() > self.max_content_length {
            page.truncate_content(self.max_content_length);
        }

        // Generate extract (first paragraph or meta description)
        page.extract = page.meta_description.clone().or_else(|| {
            page.paragraphs.first().cloned()
        });

        page
    }
} 