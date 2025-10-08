use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Idle,
    Syncing,
    Parsing,
    Importing,
    BuildingThreads,
    Completed,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetrics {
    pub emails_parsed: usize,
    pub parse_errors: usize,
    pub authors_imported: usize,
    pub emails_imported: usize,
    pub threads_created: usize,
}

impl Default for SyncMetrics {
    fn default() -> Self {
        Self {
            emails_parsed: 0,
            parse_errors: 0,
            authors_imported: 0,
            emails_imported: 0,
            threads_created: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub current_step: String,
    pub phase_details: Option<String>,
    pub processed: usize,
    pub total: Option<usize>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobState {
    pub id: String,
    pub status: JobStatus,
    pub progress: JobProgress,
    pub metrics: Option<SyncMetrics>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl Default for JobState {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            status: JobStatus::Idle,
            progress: JobProgress {
                current_step: String::new(),
                phase_details: None,
                processed: 0,
                total: None,
                errors: Vec::new(),
                warnings: Vec::new(),
            },
            metrics: None,
            started_at: None,
            completed_at: None,
            error_message: None,
        }
    }
}

pub struct JobManager {
    pub state: Arc<Mutex<JobState>>,
    pub cancellation_token: CancellationToken,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(JobState::default())),
            cancellation_token: CancellationToken::new(),
        }
    }

    pub async fn get_state(&self) -> JobState {
        self.state.lock().await.clone()
    }

    pub async fn start_job(&self) -> Result<String, String> {
        let mut state = self.state.lock().await;

        // Check if already running
        if matches!(
            state.status,
            JobStatus::Syncing | JobStatus::Parsing | JobStatus::Importing | JobStatus::BuildingThreads
        ) {
            return Err("A sync job is already running".to_string());
        }

        // Reset state for new job
        let job_id = Uuid::new_v4().to_string();
        *state = JobState {
            id: job_id.clone(),
            status: JobStatus::Syncing,
            progress: JobProgress {
                current_step: "Starting sync...".to_string(),
                phase_details: None,
                processed: 0,
                total: None,
                errors: Vec::new(),
                warnings: Vec::new(),
            },
            metrics: Some(SyncMetrics::default()),
            started_at: Some(Utc::now()),
            completed_at: None,
            error_message: None,
        };

        Ok(job_id)
    }

    pub async fn update_status(&self, status: JobStatus, step: String) {
        let mut state = self.state.lock().await;
        state.status = status;
        state.progress.current_step = step;
    }

    pub async fn update_phase_details(&self, details: String) {
        let mut state = self.state.lock().await;
        state.progress.phase_details = Some(details);
    }

    pub async fn update_progress(&self, processed: usize, total: Option<usize>) {
        let mut state = self.state.lock().await;
        state.progress.processed = processed;
        state.progress.total = total;
    }

    pub async fn add_error(&self, error: String) {
        let mut state = self.state.lock().await;
        state.progress.errors.push(error);
    }

    pub async fn add_warning(&self, warning: String) {
        let mut state = self.state.lock().await;
        state.progress.warnings.push(warning);
    }

    pub async fn update_metrics<F>(&self, updater: F)
    where
        F: FnOnce(&mut SyncMetrics),
    {
        let mut state = self.state.lock().await;
        if let Some(metrics) = &mut state.metrics {
            updater(metrics);
        }
    }

    pub async fn complete_job(&self) {
        let mut state = self.state.lock().await;
        state.status = JobStatus::Completed;
        state.completed_at = Some(Utc::now());
    }

    pub async fn fail_job(&self, error: String) {
        let mut state = self.state.lock().await;
        state.status = JobStatus::Error;
        state.error_message = Some(error);
        state.completed_at = Some(Utc::now());
    }

    pub async fn cancel_job(&self) {
        self.cancellation_token.cancel();
        let mut state = self.state.lock().await;
        state.status = JobStatus::Cancelled;
        state.completed_at = Some(Utc::now());
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}
