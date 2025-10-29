use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Representation of a thread document stored in Meilisearch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ThreadDocument {
    pub thread_id: i32,
    pub mailing_list_id: i32,
    pub mailing_list: String,
    pub root_message_id: String,
    pub subject: String,
    pub normalized_subject: Option<String>,
    pub start_ts: i64,
    pub last_ts: i64,
    pub message_count: i32,
    pub discussion_text: String,
    pub participants: Vec<String>,
    pub participant_ids: Vec<i32>,
    #[serde(default)]
    pub participant_emails: Vec<String>,
    pub has_patches: bool,
    pub series_id: Option<String>,
    pub series_number: Option<i32>,
    pub series_total: Option<i32>,
    pub starter_id: i32,
    pub starter_name: Option<String>,
    pub starter_email: String,
    pub first_post_excerpt: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "_vectors",
        serialize_with = "serialize_vectors"
    )]
    pub vector: Option<Vec<f32>>,
}

fn serialize_vectors<S>(value: &Option<Vec<f32>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;

    if let Some(vector) = value {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("threads-qwen3", vector)?;
        map.end()
    } else {
        serializer.serialize_none()
    }
}

impl ThreadDocument {
    pub fn start_date(&self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(self.start_ts, 0).single()
    }

    pub fn last_date(&self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(self.last_ts, 0).single()
    }
}

/// Representation of an author document stored in Meilisearch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthorDocument {
    pub author_id: i32,
    pub canonical_name: Option<String>,
    pub email: String,
    pub aliases: Vec<String>,
    pub mailing_lists: Vec<String>,
    pub first_seen_ts: Option<i64>,
    pub last_seen_ts: Option<i64>,
    pub first_email_ts: Option<i64>,
    pub last_email_ts: Option<i64>,
    pub thread_count: i64,
    pub email_count: i64,
    pub mailing_list_stats: Vec<AuthorMailingListStats>,
}

impl AuthorDocument {
    pub fn first_seen(&self) -> Option<DateTime<Utc>> {
        self.first_seen_ts
            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
    }

    pub fn last_seen(&self) -> Option<DateTime<Utc>> {
        self.last_seen_ts
            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
    }

    pub fn first_email_date(&self) -> Option<DateTime<Utc>> {
        self.first_email_ts
            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
    }

    pub fn last_email_date(&self) -> Option<DateTime<Utc>> {
        self.last_email_ts
            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthorMailingListStats {
    pub slug: String,
    pub email_count: i64,
    pub thread_count: i64,
    pub first_email_ts: Option<i64>,
    pub last_email_ts: Option<i64>,
}

impl AuthorMailingListStats {
    pub fn first_email_date(&self) -> Option<DateTime<Utc>> {
        self.first_email_ts
            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
    }

    pub fn last_email_date(&self) -> Option<DateTime<Utc>> {
        self.last_email_ts
            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
    }
}
