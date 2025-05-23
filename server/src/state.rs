// back-end/src/state.rs
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

// Bir işin durumu:
#[derive(Clone, Serialize, Debug)]
pub enum JobStatus {
    Pending,
    Done { summary: String },
    Failed { error: String },
}

// Uygulama durumu: job_id → JobStatus
#[derive(Clone, Debug)]
pub struct AppState {
    pub jobs: Arc<DashMap<Uuid, JobStatus>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            jobs: Arc::new(DashMap::new()),
        }
    }
}
