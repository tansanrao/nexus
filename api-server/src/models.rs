use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use rocket_db_pools::sqlx::FromRow;

// ===== Mailing List Models =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MailingList {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub sync_priority: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub last_synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MailingListRepository {
    pub id: i32,
    pub mailing_list_id: i32,
    pub repo_url: String,
    pub repo_order: i32,
    pub last_indexed_commit: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailingListWithRepos {
    #[serde(flatten)]
    pub list: MailingList,
    pub repos: Vec<MailingListRepository>,
}

// ===== Core Data Models (Partitioned by mailing_list_id) =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct Author {
    pub id: i32,
    pub email: String,
    pub canonical_name: Option<String>,
    pub first_seen: Option<DateTime<Utc>>,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct AuthorNameAlias {
    pub id: i32,
    pub author_id: i32,
    pub name: String,
    pub usage_count: i32,
    pub first_seen: Option<DateTime<Utc>>,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct AuthorMailingListActivity {
    pub author_id: i32,
    pub mailing_list_id: i32,
    pub first_email_date: Option<DateTime<Utc>>,
    pub last_email_date: Option<DateTime<Utc>>,
    pub email_count: i64,
    pub thread_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct Email {
    pub id: i32,
    pub mailing_list_id: i32,
    pub message_id: String,
    pub git_commit_hash: String,
    pub author_id: i32,
    pub subject: String,
    pub date: DateTime<Utc>,
    pub in_reply_to: Option<String>,
    pub body: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Thread {
    pub id: i32,
    pub mailing_list_id: i32,
    pub root_message_id: String,
    pub subject: String,
    pub start_date: DateTime<Utc>,
    pub last_date: DateTime<Utc>,
    pub message_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct ThreadMembership {
    pub mailing_list_id: i32,
    pub thread_id: i32,
    pub email_id: i32,
    pub depth: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct EmailRecipient {
    pub id: i32,
    pub mailing_list_id: i32,
    pub email_id: i32,
    pub author_id: i32,
    pub recipient_type: Option<String>,
}

// ===== Extended Structs for API Responses =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailWithAuthor {
    pub id: i32,
    pub mailing_list_id: i32,
    pub message_id: String,
    pub git_commit_hash: String,
    pub author_id: i32,
    pub subject: String,
    pub date: DateTime<Utc>,
    pub in_reply_to: Option<String>,
    pub body: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub author_name: Option<String>,
    pub author_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadDetail {
    pub thread: Thread,
    pub emails: Vec<EmailHierarchy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailHierarchy {
    pub id: i32,
    pub mailing_list_id: i32,
    pub message_id: String,
    pub git_commit_hash: String,
    pub author_id: i32,
    pub subject: String,
    pub date: DateTime<Utc>,
    pub in_reply_to: Option<String>,
    pub body: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub author_name: Option<String>,
    pub author_email: String,
    pub depth: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorWithStats {
    pub id: i32,
    pub email: String,
    pub canonical_name: Option<String>,
    pub first_seen: Option<DateTime<Utc>>,
    pub last_seen: Option<DateTime<Utc>>,
    pub email_count: i64,
    pub thread_count: i64,
    pub first_email_date: Option<DateTime<Utc>>,
    pub last_email_date: Option<DateTime<Utc>>,
    pub mailing_lists: Vec<String>,  // List of mailing list slugs
    pub name_variations: Vec<String>, // List of name variations used
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Stats {
    pub total_emails: i64,
    pub total_threads: i64,
    pub total_authors: i64,
    pub date_range_start: Option<DateTime<Utc>>,
    pub date_range_end: Option<DateTime<Utc>>,
}

// ===== Search Types =====

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchType {
    Subject,
    FullText,
}

impl Default for SearchType {
    fn default() -> Self {
        SearchType::Subject
    }
}

// ===== Author Thread Participation =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ThreadWithStarter {
    pub id: i32,
    pub mailing_list_id: i32,
    pub root_message_id: String,
    pub subject: String,
    pub start_date: DateTime<Utc>,
    pub last_date: DateTime<Utc>,
    pub message_count: Option<i32>,
    pub starter_id: i32,
    pub starter_name: Option<String>,
    pub starter_email: String,
}
