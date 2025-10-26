use crate::models::PatchType;
use crate::search::models::{AuthorDocument, AuthorMailingListStats, ThreadDocument};
use crate::search::{SearchError, SearchService};
use crate::sync::queue::JobQueue;
use chrono::{DateTime, Utc};
use log::{debug, warn};
use rocket_db_pools::sqlx::{self, PgPool};
use std::collections::{BTreeSet, HashMap, HashSet};

const THREAD_BATCH_SIZE: i64 = 200;
const DISCUSSION_REPLY_LIMIT: usize = 5;
const DISCUSSION_CHAR_LIMIT: usize = 16_000;
const DISCUSSION_EXCERPT_LIMIT: usize = 400;

pub async fn reindex_threads(
    pool: &PgPool,
    search: &SearchService,
    mailing_list_id: Option<i32>,
    job_context: Option<(&JobQueue, i32)>,
) -> Result<usize, SearchError> {
    search.ensure_thread_index().await?;

    if let Some(list_id) = mailing_list_id {
        search.delete_threads_by_mailing_list(list_id).await?;
    }

    let mut last_id: i32 = 0;
    let mut total_threads_processed: usize = 0;

    if let Some((queue, job_id)) = job_context {
        if let Err(err) = queue.heartbeat(job_id).await {
            warn!("job {}: failed to record heartbeat: {}", job_id, err);
        }
    }

    loop {
        let thread_rows: Vec<ThreadRow> = sqlx::query_as::<_, ThreadRow>(THREAD_QUERY)
            .bind(last_id)
            .bind(mailing_list_id)
            .bind(THREAD_BATCH_SIZE)
            .fetch_all(pool)
            .await
            .map_err(SearchError::Database)?;

        if thread_rows.is_empty() {
            break;
        }

        let thread_ids: Vec<i32> = thread_rows.iter().map(|row| row.id).collect();
        debug!(
            "reindex_threads: processing batch ({} threads, id range {}-{})",
            thread_rows.len(),
            thread_rows.first().map(|row| row.id).unwrap_or_default(),
            thread_rows.last().map(|row| row.id).unwrap_or_default()
        );

        let email_rows = fetch_emails(pool, &thread_ids).await?;
        let email_map = group_emails_by_thread(email_rows);

        let mut documents = Vec::with_capacity(thread_rows.len());
        for row in thread_rows.iter() {
            let emails = email_map.get(&row.id).cloned().unwrap_or_else(Vec::new);
            let document = build_thread_document(search, row, emails).await?;
            documents.push(document);
        }

        search.upsert_threads(&documents).await?;
        total_threads_processed += documents.len();
        last_id = thread_rows.last().map(|row| row.id).unwrap_or(last_id);

        if let Some((queue, job_id)) = job_context {
            if let Err(err) = queue.heartbeat(job_id).await {
                warn!("job {}: failed to record heartbeat: {}", job_id, err);
            }
        }
    }

    Ok(total_threads_processed)
}

pub async fn reindex_authors(
    pool: &PgPool,
    search: &SearchService,
    job_context: Option<(&JobQueue, i32)>,
) -> Result<usize, SearchError> {
    search.ensure_author_index().await?;

    let activity_rows: Vec<AuthorActivityRow> =
        sqlx::query_as::<_, AuthorActivityRow>(AUTHOR_ACTIVITY_QUERY)
            .fetch_all(pool)
            .await
            .map_err(SearchError::Database)?;

    if activity_rows.is_empty() {
        if let Some((queue, job_id)) = job_context {
            if let Err(err) = queue.heartbeat(job_id).await {
                warn!("job {}: failed to record heartbeat: {}", job_id, err);
            }
        }
        return Ok(0);
    }

    let alias_rows: Vec<AuthorAliasRow> = sqlx::query_as::<_, AuthorAliasRow>(AUTHOR_ALIAS_QUERY)
        .fetch_all(pool)
        .await
        .map_err(SearchError::Database)?;

    let mut alias_map: HashMap<i32, Vec<String>> = HashMap::new();
    for alias in alias_rows {
        alias_map
            .entry(alias.author_id)
            .or_default()
            .push(alias.name);
    }

    let mut author_map: HashMap<i32, AuthorDocumentBuilder> = HashMap::new();
    for row in activity_rows {
        let entry = author_map
            .entry(row.author_id)
            .or_insert_with(|| AuthorDocumentBuilder {
                author_id: row.author_id,
                email: row.email.clone(),
                canonical_name: row.canonical_name.clone(),
                first_seen: row.first_seen,
                last_seen: row.last_seen,
                mailing_lists: BTreeSet::new(),
                total_email_count: 0,
                total_thread_count: 0,
                first_email_ts: None,
                last_email_ts: None,
                per_list: Vec::new(),
            });

        entry.mailing_lists.insert(row.slug.clone());
        entry.total_email_count += row.email_count;
        entry.total_thread_count += row.thread_count;
        entry.first_email_ts = combine_min(
            entry.first_email_ts,
            row.first_email_date.map(|dt| dt.timestamp()),
        );
        entry.last_email_ts = combine_max(
            entry.last_email_ts,
            row.last_email_date.map(|dt| dt.timestamp()),
        );
        entry.per_list.push(AuthorMailingListStats {
            slug: row.slug,
            email_count: row.email_count,
            thread_count: row.thread_count,
            first_email_ts: row.first_email_date.map(|dt| dt.timestamp()),
            last_email_ts: row.last_email_date.map(|dt| dt.timestamp()),
        });
    }

    let mut documents = Vec::with_capacity(author_map.len());
    for (author_id, builder) in author_map.into_iter() {
        let aliases = alias_map.remove(&author_id).unwrap_or_default();
        documents.push(builder.into_document(aliases));
    }

    let processed = documents.len();

    search.upsert_authors(&documents).await?;

    if let Some((queue, job_id)) = job_context {
        if let Err(err) = queue.heartbeat(job_id).await {
            warn!("job {}: failed to record heartbeat: {}", job_id, err);
        }
    }

    Ok(processed)
}

fn combine_min(current: Option<i64>, candidate: Option<i64>) -> Option<i64> {
    match (current, candidate) {
        (Some(cur), Some(val)) => Some(cur.min(val)),
        (None, Some(val)) => Some(val),
        (existing, None) => existing,
    }
}

fn combine_max(current: Option<i64>, candidate: Option<i64>) -> Option<i64> {
    match (current, candidate) {
        (Some(cur), Some(val)) => Some(cur.max(val)),
        (None, Some(val)) => Some(val),
        (existing, None) => existing,
    }
}

async fn fetch_emails(
    pool: &PgPool,
    thread_ids: &[i32],
) -> Result<Vec<ThreadEmailRow>, SearchError> {
    if thread_ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query_as::<_, ThreadEmailRow>(THREAD_EMAIL_QUERY)
        .bind(thread_ids)
        .fetch_all(pool)
        .await
        .map_err(SearchError::Database)?;

    Ok(rows)
}

async fn build_thread_document(
    search: &SearchService,
    thread: &ThreadRow,
    emails: Vec<ThreadEmailRow>,
) -> Result<ThreadDocument, SearchError> {
    let mut participant_ids = Vec::new();
    let mut participant_names = Vec::new();
    let mut seen_participants: HashSet<i32> = HashSet::new();
    let mut has_patches = false;

    let mut ordered_emails = emails;
    ordered_emails.sort_by_key(|email| (email.thread_id, email.date));

    for email in &ordered_emails {
        if seen_participants.insert(email.author_id) {
            participant_ids.push(email.author_id);
            participant_names.push(
                email
                    .author_name
                    .clone()
                    .unwrap_or_else(|| email.author_email.clone()),
            );
        }
        if email.is_patch_only || email.patch_type.is_some() {
            has_patches = true;
        }
    }

    let (discussion_text, first_post_excerpt) = build_discussion_text(thread, &ordered_emails);

    let mut embed_text = thread
        .normalized_subject
        .clone()
        .unwrap_or_else(|| thread.subject.clone());
    if !discussion_text.is_empty() {
        embed_text.push_str("\n\n");
        embed_text.push_str(&discussion_text);
    }

    let embed_text_trimmed = embed_text.trim();
    let vector = if embed_text_trimmed.is_empty() {
        Some(vec![0.0; search.thread_embedding_dimensions()])
    } else {
        let embedding = search
            .embed_with_fallback(
                embed_text_trimmed,
                search.thread_embedding_dimensions(),
                &format!("thread {}", thread.id),
            )
            .await?;
        Some(embedding)
    };

    Ok(ThreadDocument {
        thread_id: thread.id,
        mailing_list_id: thread.mailing_list_id,
        mailing_list: thread.mailing_list_slug.clone(),
        root_message_id: thread.root_message_id.clone(),
        subject: thread.subject.clone(),
        normalized_subject: thread.normalized_subject.clone(),
        start_ts: thread.start_date.timestamp(),
        last_ts: thread.last_date.timestamp(),
        message_count: thread.message_count,
        discussion_text,
        participants: participant_names,
        participant_ids,
        has_patches,
        series_id: thread.series_id.clone(),
        series_number: thread.series_number,
        series_total: thread.series_total,
        starter_id: thread.starter_id,
        starter_name: thread.starter_name.clone(),
        starter_email: thread.starter_email.clone(),
        first_post_excerpt,
        vector,
    })
}

fn build_discussion_text(
    thread: &ThreadRow,
    emails: &[ThreadEmailRow],
) -> (String, Option<String>) {
    if emails.is_empty() {
        return (String::new(), None);
    }

    let mut sections: Vec<String> = Vec::new();
    let mut replies_added = 0;
    let mut root_excerpt = None;
    let mut root_added = false;

    for email in emails {
        let text = email
            .search_body
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty());

        let mut body = match text {
            Some(value) => value.to_string(),
            None => continue,
        };

        if email.message_id == thread.root_message_id && !root_added {
            if body.chars().count() > DISCUSSION_CHAR_LIMIT {
                body = body.chars().take(DISCUSSION_CHAR_LIMIT).collect::<String>();
            }
            root_excerpt = Some(body.chars().take(DISCUSSION_EXCERPT_LIMIT).collect());
            sections.insert(0, body);
            root_added = true;
        } else if replies_added < DISCUSSION_REPLY_LIMIT {
            replies_added += 1;
            sections.push(body);
        }
    }

    let mut combined = String::new();
    let mut total_chars: usize = 0;

    for (index, section) in sections.into_iter().enumerate() {
        if index > 0 {
            combined.push_str("\n\n");
            total_chars = total_chars.saturating_add(2);
        }

        let chars: Vec<char> = section.chars().collect();
        if total_chars + chars.len() > DISCUSSION_CHAR_LIMIT {
            let remaining = DISCUSSION_CHAR_LIMIT.saturating_sub(total_chars);
            combined.extend(chars.into_iter().take(remaining));
            break;
        } else {
            total_chars += chars.len();
            for ch in chars {
                combined.push(ch);
            }
        }
    }

    if root_excerpt.is_none() && !combined.is_empty() {
        root_excerpt = Some(combined.chars().take(DISCUSSION_EXCERPT_LIMIT).collect());
    }

    (combined, root_excerpt)
}

fn group_emails_by_thread(rows: Vec<ThreadEmailRow>) -> HashMap<i32, Vec<ThreadEmailRow>> {
    let mut map: HashMap<i32, Vec<ThreadEmailRow>> = HashMap::new();
    for row in rows {
        map.entry(row.thread_id).or_default().push(row);
    }
    map
}

struct AuthorDocumentBuilder {
    author_id: i32,
    email: String,
    canonical_name: Option<String>,
    first_seen: Option<DateTime<Utc>>,
    last_seen: Option<DateTime<Utc>>,
    mailing_lists: BTreeSet<String>,
    total_email_count: i64,
    total_thread_count: i64,
    first_email_ts: Option<i64>,
    last_email_ts: Option<i64>,
    per_list: Vec<AuthorMailingListStats>,
}

impl AuthorDocumentBuilder {
    fn into_document(self, aliases: Vec<String>) -> AuthorDocument {
        AuthorDocument {
            author_id: self.author_id,
            canonical_name: self.canonical_name,
            email: self.email,
            aliases,
            mailing_lists: self.mailing_lists.into_iter().collect(),
            first_seen_ts: self.first_seen.map(|dt| dt.timestamp()),
            last_seen_ts: self.last_seen.map(|dt| dt.timestamp()),
            first_email_ts: self.first_email_ts,
            last_email_ts: self.last_email_ts,
            thread_count: self.total_thread_count,
            email_count: self.total_email_count,
            mailing_list_stats: self.per_list,
        }
    }
}

#[derive(sqlx::FromRow, Clone)]
struct ThreadRow {
    id: i32,
    mailing_list_id: i32,
    mailing_list_slug: String,
    root_message_id: String,
    subject: String,
    normalized_subject: Option<String>,
    start_date: DateTime<Utc>,
    last_date: DateTime<Utc>,
    message_count: i32,
    starter_id: i32,
    starter_name: Option<String>,
    starter_email: String,
    series_id: Option<String>,
    series_number: Option<i32>,
    series_total: Option<i32>,
}

#[derive(sqlx::FromRow, Clone)]
struct ThreadEmailRow {
    thread_id: i32,
    message_id: String,
    date: DateTime<Utc>,
    search_body: Option<String>,
    is_patch_only: bool,
    patch_type: Option<PatchType>,
    author_id: i32,
    author_name: Option<String>,
    author_email: String,
}

#[derive(sqlx::FromRow, Clone)]
struct AuthorActivityRow {
    author_id: i32,
    email: String,
    canonical_name: Option<String>,
    first_seen: Option<DateTime<Utc>>,
    last_seen: Option<DateTime<Utc>>,
    slug: String,
    email_count: i64,
    thread_count: i64,
    first_email_date: Option<DateTime<Utc>>,
    last_email_date: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct AuthorAliasRow {
    author_id: i32,
    name: String,
}

const THREAD_QUERY: &str = r#"
    SELECT
        t.id,
        t.mailing_list_id,
        ml.slug AS mailing_list_slug,
        t.root_message_id,
        t.subject,
        starter.normalized_subject,
        t.start_date,
        t.last_date,
        COALESCE(t.message_count, 0) AS message_count,
        starter.author_id AS starter_id,
        starter_author.canonical_name AS starter_name,
        starter_author.email AS starter_email,
        starter.series_id,
        starter.series_number,
        starter.series_total
    FROM threads t
    JOIN mailing_lists ml ON ml.id = t.mailing_list_id
    JOIN emails starter ON starter.message_id = t.root_message_id
        AND starter.mailing_list_id = t.mailing_list_id
    JOIN authors starter_author ON starter.author_id = starter_author.id
    WHERE t.id > $1
      AND ($2::int IS NULL OR t.mailing_list_id = $2)
    ORDER BY t.id
    LIMIT $3
"#;

const THREAD_EMAIL_QUERY: &str = r#"
    SELECT
        tm.thread_id,
        e.message_id,
        e.date,
        e.search_body,
        e.is_patch_only,
        e.patch_type,
        e.author_id,
        a.canonical_name AS author_name,
        a.email AS author_email
    FROM thread_memberships tm
    JOIN emails e ON e.id = tm.email_id
    JOIN authors a ON e.author_id = a.id
    WHERE tm.thread_id = ANY($1)
    ORDER BY tm.thread_id, e.date
"#;

const AUTHOR_ACTIVITY_QUERY: &str = r#"
    SELECT
        act.author_id,
        a.email,
        a.canonical_name,
        a.first_seen,
        a.last_seen,
        ml.slug,
        act.email_count,
        act.thread_count,
        act.first_email_date,
        act.last_email_date
    FROM author_mailing_list_activity act
    JOIN authors a ON a.id = act.author_id
    JOIN mailing_lists ml ON ml.id = act.mailing_list_id
"#;

const AUTHOR_ALIAS_QUERY: &str = r#"
    SELECT
        author_id,
        name
    FROM author_name_aliases
    ORDER BY author_id, usage_count DESC
"#;
