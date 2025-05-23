// back-end/src/handlers.rs
use axum::{
    extract::{ Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
    Extension
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Stdio;
use tokio::process::Command;
use uuid::Uuid;

use scraper_service_cli::{
    scraper::Scraper,
    ai::AIAnalyzer,
};

use crate::state::{AppState, JobStatus};

#[derive(Deserialize)]
pub struct ScrapeUrlPayload {
    pub url: String,
}

#[derive(Serialize)]
pub struct JobCreated {
    pub job_id: Uuid,
}

#[derive(Serialize)]
pub struct JobPollResponse {
    pub status: String,
    pub summary: Option<String>,
    pub error: Option<String>,
}

pub async fn scrape_url(
    Extension(state): Extension<AppState>,
    Json(payload): Json<ScrapeUrlPayload>,
) -> impl IntoResponse {
    // 1) Yeni job_id üret, Pending olarak kaydet
    let job_id = Uuid::new_v4();
    state.jobs.insert(job_id, JobStatus::Pending);

    // 2) Arka plana işi spawn et
    let state_clone = state.clone();
    let url = payload.url.clone();
    tokio::spawn(async move {
        // A) spawn_blocking ile scraping
        let pages = match tokio::task::spawn_blocking(move || {
            // Bu closure tamamen sync kod
            //  - Scraper::new
            //  - scraper.scrape block_on
            let mut scraper = Scraper::new(&url, 100, 5, 10_000)
                .map_err(|e| format!("Init error: {}", e))?;
            // block_on ile async scrape çalıştır
            futures::executor::block_on(scraper.scrape(&url))
                .map_err(|e| format!("Scrape error: {}", e))
        })
        .await
        {
            Ok(Ok(p)) if !p.is_empty() => p,
            Ok(Ok(_)) => {
                state_clone.jobs.insert(job_id, JobStatus::Failed {
                    error: "No pages scraped".into(),
                });
                return;
            }
            Ok(Err(e)) => {
                state_clone.jobs.insert(job_id, JobStatus::Failed {
                    error: e,
                });
                return;
            }
            Err(e) => {
                state_clone.jobs.insert(job_id, JobStatus::Failed {
                    error: format!("Thread join error: {}", e),
                });
                return;
            }
        };

        // B) Async AI analizi
        let api_key = match std::env::var("OPENAI_API_KEY") {
            Ok(k) => k,
            Err(_) => {
                state_clone.jobs.insert(job_id, JobStatus::Failed {
                    error: "Missing OPENAI_API_KEY".into(),
                });
                return;
            }
        };
        let summary = match AIAnalyzer::new(api_key).analyze_pages(&pages).await {
            Ok(text) => text,
            Err(e) => {
                state_clone.jobs.insert(job_id, JobStatus::Failed {
                    error: format!("AI error: {}", e),
                });
                return;
            }
        };

        // C) Başarı durumu
        state_clone.jobs.insert(job_id, JobStatus::Done { summary });
    });

    // 3) Hemen 202 ve job_id dön
    (StatusCode::ACCEPTED, Json(json!(JobCreated { job_id })))
}
/// GET /api/jobs/:id
pub async fn poll_job(
    Extension(state): Extension<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    if let Some(status) = state.jobs.get(&id) {
        // 4) Status'a göre yanıtı oluştur
        let resp = match status.value() {
            JobStatus::Pending => JobPollResponse {
                status: "pending".to_string(),
                summary: None,
                error: None,
            },
            JobStatus::Done { summary } => JobPollResponse {
                status: "done".to_string(),
                summary: Some(summary.clone()),
                error: None,
            },
            JobStatus::Failed { error } => JobPollResponse {
                status: "failed".to_string(),
                summary: None,
                error: Some(error.clone()),
            },
        };
        (StatusCode::OK, Json(json!(resp)))
    } else {
        // Job bulunamazsa 404
        (StatusCode::NOT_FOUND, Json(json!({ "error": "Job not found" })))
    }
}
