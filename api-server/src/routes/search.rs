use crate::auth::RequireAdmin;
use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::{
    ApiResponse, AuthorSearchHit, AuthorSearchMailingListStats, AuthorSearchPage, PaginationMeta,
    ResponseMeta, SortDescriptor, SortDirection, ThreadSearchHighlights, ThreadSearchHit,
    ThreadSearchPage, ThreadSearchParticipant, ThreadSearchScore, ThreadSearchThreadSummary,
};
use crate::routes::helpers::{resolve_mailing_list_id, resolve_mailing_list_ids};
use crate::routes::params::{AuthorSearchParams, AuthorSortField, SortOrder, ThreadSearchParams};
use crate::search::{
    AuthorHit, AuthorSearchPayload, AuthorSearchResults, SearchService, ThreadHit,
    ThreadMailingListFilter, ThreadSearchPayload, ThreadSearchResults,
};
use crate::sync::queue::{JobQueue, JobRecord, JobType};
use chrono::{DateTime, SecondsFormat, Utc};
use rocket::serde::json::Json;
use rocket::{State, get, post};
use rocket_db_pools::{Connection, sqlx};
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::Deserialize;
use serde_json::{Map as JsonMap, Value as JsonValue, json};

#[openapi(tag = "Search")]
#[get("/lists/<slug>/threads/search?<params..>")]
pub async fn search_threads_for_list(
    slug: String,
    params: ThreadSearchParams,
    search: &State<SearchService>,
    mut db: Connection<NexusDb>,
) -> Result<Json<ApiResponse<ThreadSearchPage>>, ApiError> {
    let query = params
        .query()
        .ok_or_else(|| ApiError::BadRequest("Query parameter 'q' is required".to_string()))?
        .to_string();

    let page = params.page();
    let size = params.size();
    let start_date = params.start_date_utc();
    let end_date = params.end_date_utc();

    if let (Some(start), Some(end)) = (start_date, end_date) {
        if end < start {
            return Err(ApiError::BadRequest(
                "endDate must be greater than or equal to startDate".to_string(),
            ));
        }
    }

    let semantic_ratio = params
        .semantic_ratio()
        .unwrap_or_else(|| search.default_semantic_ratio());

    let participant_ids = params.participant_ids();
    let series_id = params.series_id();
    let has_patches = params.has_patches();
    let starter_id = params.starter_id();

    let sort_fields = params.sort_fields();
    let (sort_meta, sort_expressions) = parse_thread_search_sorts(&sort_fields);

    let mailing_list_id = resolve_mailing_list_id(&slug, &mut db).await?;

    let payload = ThreadSearchPayload {
        query: query.clone(),
        page,
        size,
        semantic_ratio,
        start_date,
        end_date,
        has_patches,
        starter_id,
        participant_ids: participant_ids.clone(),
        series_id: series_id.clone(),
        mailing_lists: vec![ThreadMailingListFilter {
            slug: slug.clone(),
            mailing_list_id: Some(mailing_list_id),
        }],
        sort_expressions,
    };

    let ThreadSearchResults {
        hits: raw_hits,
        total,
    } = search.search_threads(payload).await?;
    let hits = map_thread_hits(raw_hits, semantic_ratio);

    let pagination = PaginationMeta::new(page, size, total);

    let mut meta = ResponseMeta::default()
        .with_list_id(slug.clone())
        .with_pagination(pagination)
        .with_sort(sort_meta);

    let mut filters = JsonMap::new();
    if let Some(start) = start_date {
        filters.insert(
            "startDate".to_string(),
            JsonValue::String(start.to_rfc3339_opts(SecondsFormat::Secs, true)),
        );
    }
    if let Some(end) = end_date {
        filters.insert(
            "endDate".to_string(),
            JsonValue::String(end.to_rfc3339_opts(SecondsFormat::Secs, true)),
        );
    }
    if let Some(value) = has_patches {
        filters.insert("hasPatches".to_string(), JsonValue::Bool(value));
    }
    if let Some(value) = starter_id {
        filters.insert("starterId".to_string(), JsonValue::from(value));
    }
    if !participant_ids.is_empty() {
        filters.insert(
            "participantId".to_string(),
            JsonValue::Array(
                participant_ids
                    .iter()
                    .map(|id| JsonValue::from(*id))
                    .collect(),
            ),
        );
    }
    if let Some(value) = series_id.clone() {
        filters.insert("seriesId".to_string(), JsonValue::String(value));
    }

    if !filters.is_empty() {
        meta = meta.with_filters(filters);
    }

    let mut search_meta = JsonMap::new();
    search_meta.insert("query".to_string(), JsonValue::String(query.clone()));
    search_meta.insert("page".to_string(), JsonValue::from(page));
    search_meta.insert("pageSize".to_string(), JsonValue::from(size));
    search_meta.insert("semanticRatio".to_string(), JsonValue::from(semantic_ratio));
    if !participant_ids.is_empty() {
        search_meta.insert(
            "participantIds".to_string(),
            JsonValue::Array(
                participant_ids
                    .iter()
                    .map(|id| JsonValue::from(*id))
                    .collect(),
            ),
        );
    }
    if let Some(value) = series_id {
        search_meta.insert("seriesId".to_string(), JsonValue::String(value));
    }
    if let Some(value) = has_patches {
        search_meta.insert("hasPatches".to_string(), JsonValue::Bool(value));
    }
    if let Some(value) = starter_id {
        search_meta.insert("starterId".to_string(), JsonValue::from(value));
    }
    if let Some(value) = start_date {
        search_meta.insert(
            "startDate".to_string(),
            JsonValue::String(value.to_rfc3339_opts(SecondsFormat::Secs, true)),
        );
    }
    if let Some(value) = end_date {
        search_meta.insert(
            "endDate".to_string(),
            JsonValue::String(value.to_rfc3339_opts(SecondsFormat::Secs, true)),
        );
    }

    let mut extra = JsonMap::new();
    extra.insert("search".to_string(), JsonValue::Object(search_meta));
    meta = meta.with_extra(extra);

    Ok(Json(ApiResponse::with_meta(
        ThreadSearchPage { hits, total },
        meta,
    )))
}

#[openapi(tag = "Search")]
#[get("/search/threads?<params..>")]
pub async fn search_threads_global(
    params: Option<ThreadSearchParams>,
    search: &State<SearchService>,
    mut db: Connection<NexusDb>,
) -> Result<Json<ApiResponse<ThreadSearchPage>>, ApiError> {
    if !search.allow_global_thread_search() {
        return Err(ApiError::BadRequest(
            "Global thread search is disabled for this deployment".to_string(),
        ));
    }

    let params = params.unwrap_or_default();
    let query = params
        .query()
        .ok_or_else(|| ApiError::BadRequest("Query parameter 'q' is required".to_string()))?
        .to_string();

    let page = params.page();
    let size = params.size();
    let start_date = params.start_date_utc();
    let end_date = params.end_date_utc();

    if let (Some(start), Some(end)) = (start_date, end_date) {
        if end < start {
            return Err(ApiError::BadRequest(
                "endDate must be greater than or equal to startDate".to_string(),
            ));
        }
    }

    let semantic_ratio = params
        .semantic_ratio()
        .unwrap_or_else(|| search.default_semantic_ratio());

    let participant_ids = params.participant_ids();
    let series_id = params.series_id();
    let has_patches = params.has_patches();
    let starter_id = params.starter_id();

    let sort_fields = params.sort_fields();
    let (sort_meta, sort_expressions) = parse_thread_search_sorts(&sort_fields);

    let mailing_lists_sanitized = params.mailing_lists();
    let mailing_filters = if mailing_lists_sanitized.is_empty() {
        Vec::new()
    } else {
        resolve_mailing_list_ids(&mailing_lists_sanitized, &mut db)
            .await?
            .into_iter()
            .map(|(slug, id)| ThreadMailingListFilter {
                slug,
                mailing_list_id: Some(id),
            })
            .collect()
    };

    let payload = ThreadSearchPayload {
        query: query.clone(),
        page,
        size,
        semantic_ratio,
        start_date,
        end_date,
        has_patches,
        starter_id,
        participant_ids: participant_ids.clone(),
        series_id: series_id.clone(),
        mailing_lists: mailing_filters,
        sort_expressions,
    };

    let ThreadSearchResults {
        hits: raw_hits,
        total,
    } = search.search_threads(payload).await?;
    let hits = map_thread_hits(raw_hits, semantic_ratio);

    let pagination = PaginationMeta::new(page, size, total);

    let mut meta = ResponseMeta::default()
        .with_pagination(pagination)
        .with_sort(sort_meta);

    let mut filters = JsonMap::new();
    if let Some(start) = start_date {
        filters.insert(
            "startDate".to_string(),
            JsonValue::String(start.to_rfc3339_opts(SecondsFormat::Secs, true)),
        );
    }
    if let Some(end) = end_date {
        filters.insert(
            "endDate".to_string(),
            JsonValue::String(end.to_rfc3339_opts(SecondsFormat::Secs, true)),
        );
    }
    if let Some(value) = has_patches {
        filters.insert("hasPatches".to_string(), JsonValue::Bool(value));
    }
    if let Some(value) = starter_id {
        filters.insert("starterId".to_string(), JsonValue::from(value));
    }
    if !participant_ids.is_empty() {
        filters.insert(
            "participantId".to_string(),
            JsonValue::Array(
                participant_ids
                    .iter()
                    .map(|id| JsonValue::from(*id))
                    .collect(),
            ),
        );
    }
    if let Some(value) = series_id.clone() {
        filters.insert("seriesId".to_string(), JsonValue::String(value));
    }
    if !mailing_lists_sanitized.is_empty() {
        filters.insert(
            "mailingList".to_string(),
            JsonValue::Array(
                mailing_lists_sanitized
                    .iter()
                    .map(|slug| JsonValue::String(slug.clone()))
                    .collect(),
            ),
        );
    }

    if !filters.is_empty() {
        meta = meta.with_filters(filters);
    }

    let mut search_meta = JsonMap::new();
    search_meta.insert("query".to_string(), JsonValue::String(query.clone()));
    search_meta.insert("page".to_string(), JsonValue::from(page));
    search_meta.insert("pageSize".to_string(), JsonValue::from(size));
    search_meta.insert("semanticRatio".to_string(), JsonValue::from(semantic_ratio));
    if !participant_ids.is_empty() {
        search_meta.insert(
            "participantIds".to_string(),
            JsonValue::Array(
                participant_ids
                    .iter()
                    .map(|id| JsonValue::from(*id))
                    .collect(),
            ),
        );
    }
    if let Some(value) = series_id {
        search_meta.insert("seriesId".to_string(), JsonValue::String(value));
    }
    if let Some(value) = has_patches {
        search_meta.insert("hasPatches".to_string(), JsonValue::Bool(value));
    }
    if let Some(value) = starter_id {
        search_meta.insert("starterId".to_string(), JsonValue::from(value));
    }
    if let Some(value) = start_date {
        search_meta.insert(
            "startDate".to_string(),
            JsonValue::String(value.to_rfc3339_opts(SecondsFormat::Secs, true)),
        );
    }
    if let Some(value) = end_date {
        search_meta.insert(
            "endDate".to_string(),
            JsonValue::String(value.to_rfc3339_opts(SecondsFormat::Secs, true)),
        );
    }
    if !mailing_lists_sanitized.is_empty() {
        search_meta.insert(
            "mailingLists".to_string(),
            JsonValue::Array(
                mailing_lists_sanitized
                    .into_iter()
                    .map(JsonValue::String)
                    .collect(),
            ),
        );
    }

    let mut extra = JsonMap::new();
    extra.insert("search".to_string(), JsonValue::Object(search_meta));
    meta = meta.with_extra(extra);

    Ok(Json(ApiResponse::with_meta(
        ThreadSearchPage { hits, total },
        meta,
    )))
}

#[openapi(tag = "Search")]
#[get("/authors/search?<params..>")]
pub async fn search_authors(
    params: Option<AuthorSearchParams>,
    search: &State<SearchService>,
    mut db: Connection<NexusDb>,
) -> Result<Json<ApiResponse<AuthorSearchPage>>, ApiError> {
    let params = params.unwrap_or_default();
    let page = params.page();
    let size = params.size();
    let sort_order = params.order;
    let sort_field = params.sort_by;

    let (sort_descriptor, sort_expression) = author_sort_descriptor(sort_field, sort_order);

    let mailing_lists_sanitized = params.mailing_lists();
    if !mailing_lists_sanitized.is_empty() {
        // Validate mailing lists exist; IDs are unused but validation ensures quick feedback.
        let _ = resolve_mailing_list_ids(&mailing_lists_sanitized, &mut db).await?;
    }

    let payload = AuthorSearchPayload {
        query: params.raw_query(),
        page,
        size,
        sort_expression,
        mailing_lists: mailing_lists_sanitized.clone(),
    };

    let AuthorSearchResults {
        hits: raw_hits,
        total,
    } = search.search_authors(payload).await?;
    let hits = map_author_hits(raw_hits);

    let pagination = PaginationMeta::new(page, size, total);

    let mut meta = ResponseMeta::default()
        .with_pagination(pagination)
        .with_sort(vec![sort_descriptor]);

    let mut filters = JsonMap::new();
    if !mailing_lists_sanitized.is_empty() {
        filters.insert(
            "mailingList".to_string(),
            JsonValue::Array(
                mailing_lists_sanitized
                    .iter()
                    .map(|slug| JsonValue::String(slug.clone()))
                    .collect(),
            ),
        );
    }

    if !filters.is_empty() {
        meta = meta.with_filters(filters);
    }

    let mut search_meta = JsonMap::new();
    if let Some(query) = params.raw_query() {
        search_meta.insert("query".to_string(), JsonValue::String(query));
    }
    search_meta.insert("page".to_string(), JsonValue::from(page));
    search_meta.insert("pageSize".to_string(), JsonValue::from(size));
    search_meta.insert(
        "sortBy".to_string(),
        JsonValue::String(author_sort_field_name(sort_field).to_string()),
    );
    search_meta.insert(
        "sortOrder".to_string(),
        JsonValue::String(
            match sort_order {
                SortOrder::Asc => "asc",
                SortOrder::Desc => "desc",
            }
            .to_string(),
        ),
    );
    if !mailing_lists_sanitized.is_empty() {
        search_meta.insert(
            "mailingLists".to_string(),
            JsonValue::Array(
                mailing_lists_sanitized
                    .into_iter()
                    .map(JsonValue::String)
                    .collect(),
            ),
        );
    }

    let mut extra = JsonMap::new();
    extra.insert("search".to_string(), JsonValue::Object(search_meta));
    meta = meta.with_extra(extra);

    Ok(Json(ApiResponse::with_meta(
        AuthorSearchPage { hits, total },
        meta,
    )))
}

#[derive(Debug, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexRefreshRequest {
    #[serde(default)]
    pub mailing_list_slug: Option<String>,
    #[serde(default)]
    pub priority: Option<i32>,
}

#[derive(Debug, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexResetRequest {
    #[serde(default)]
    pub priority: Option<i32>,
}

#[openapi(tag = "Admin - Search")]
#[post("/search/indexes/threads/refresh", data = "<request>")]
pub async fn refresh_search_indexes(
    _admin: RequireAdmin,
    request: Option<Json<SearchIndexRefreshRequest>>,
    pool: &State<sqlx::PgPool>,
) -> Result<Json<ApiResponse<JobRecord>>, ApiError> {
    let payload = request.map(|json| json.into_inner()).unwrap_or_default();
    let priority = payload.priority.unwrap_or(0);

    let queue = JobQueue::new(pool.inner().clone());

    let job_id = queue
        .enqueue_job(
            JobType::IndexMaintenance,
            None,
            json!({
                "action": "refresh",
                "mailingListSlug": payload.mailing_list_slug,
            }),
            priority,
        )
        .await?;

    let job = queue
        .get_job(job_id)
        .await?
        .ok_or_else(|| ApiError::InternalError("Failed to fetch queued job".to_string()))?;

    Ok(Json(ApiResponse::with_meta(job, ResponseMeta::default())))
}

#[openapi(tag = "Admin - Search")]
#[post("/search/indexes/reset", data = "<request>")]
pub async fn reset_search_indexes(
    _admin: RequireAdmin,
    request: Option<Json<SearchIndexResetRequest>>,
    pool: &State<sqlx::PgPool>,
) -> Result<Json<ApiResponse<JobRecord>>, ApiError> {
    let payload = request.map(|json| json.into_inner()).unwrap_or_default();
    let priority = payload.priority.unwrap_or(0);

    let queue = JobQueue::new(pool.inner().clone());

    let job_id = queue
        .enqueue_job(
            JobType::IndexMaintenance,
            None,
            json!({ "action": "reset" }),
            priority,
        )
        .await?;

    let job = queue
        .get_job(job_id)
        .await?
        .ok_or_else(|| ApiError::InternalError("Failed to fetch queued job".to_string()))?;

    Ok(Json(ApiResponse::with_meta(job, ResponseMeta::default())))
}

fn parse_thread_search_sorts(values: &[String]) -> (Vec<SortDescriptor>, Vec<String>) {
    if values.is_empty() {
        return (
            vec![SortDescriptor {
                field: "lastActivity".to_string(),
                direction: SortDirection::Desc,
            }],
            vec!["last_ts:desc".to_string()],
        );
    }

    let mut descriptors = Vec::new();
    let mut expressions = Vec::new();

    for value in values {
        let mut parts = value.splitn(2, ':');
        let field = parts.next().unwrap_or_default().trim();
        if field.is_empty() {
            continue;
        }
        let direction_raw = parts.next().unwrap_or("desc").trim();

        let (api_field, meili_field) = match field {
            "lastActivity" => ("lastActivity", Some("last_ts")),
            "startDate" => ("startDate", Some("start_ts")),
            "messageCount" => ("messageCount", Some("message_count")),
            "semanticScore" => ("semanticScore", None),
            _ => continue,
        };

        let sort_direction = if direction_raw.eq_ignore_ascii_case("asc") {
            SortDirection::Asc
        } else {
            SortDirection::Desc
        };

        let order_str = if matches!(sort_direction, SortDirection::Asc) {
            "asc"
        } else {
            "desc"
        };

        descriptors.push(SortDescriptor {
            field: api_field.to_string(),
            direction: sort_direction,
        });

        if let Some(meili_field) = meili_field {
            expressions.push(format!("{meili_field}:{order_str}"));
        }
    }

    if descriptors.is_empty() {
        descriptors.push(SortDescriptor {
            field: "lastActivity".to_string(),
            direction: SortDirection::Desc,
        });
        expressions.push("last_ts:desc".to_string());
    }

    (descriptors, expressions)
}

fn map_thread_hits(hits: Vec<ThreadHit>, semantic_ratio: f32) -> Vec<ThreadSearchHit> {
    hits.into_iter()
        .map(|hit| {
            let ThreadHit {
                document,
                ranking_score,
                formatted,
            } = hit;

            let start_date = document
                .start_date()
                .or_else(|| timestamp_to_datetime(document.start_ts))
                .unwrap_or_else(Utc::now);
            let last_activity = document
                .last_date()
                .or_else(|| timestamp_to_datetime(document.last_ts))
                .unwrap_or_else(Utc::now);

            ThreadSearchHit {
                thread: ThreadSearchThreadSummary {
                    thread_id: document.thread_id,
                    mailing_list_id: document.mailing_list_id,
                    mailing_list_slug: document.mailing_list.clone(),
                    root_message_id: document.root_message_id.clone(),
                    subject: document.subject.clone(),
                    normalized_subject: document.normalized_subject.clone(),
                    start_date,
                    last_activity,
                    message_count: document.message_count,
                    starter_id: document.starter_id,
                    starter_name: document.starter_name.clone(),
                    starter_email: document.starter_email.clone(),
                },
                participants: collect_participants(&document),
                has_patches: document.has_patches,
                series_id: document.series_id.clone(),
                series_number: document.series_number,
                series_total: document.series_total,
                first_post_excerpt: document.first_post_excerpt.clone(),
                score: ThreadSearchScore {
                    ranking_score,
                    semantic_ratio,
                },
                highlights: formatted
                    .as_ref()
                    .and_then(|value| build_thread_highlights(value)),
            }
        })
        .collect()
}

fn map_author_hits(hits: Vec<AuthorHit>) -> Vec<AuthorSearchHit> {
    hits.into_iter()
        .map(|hit| {
            let document = hit.document;

            AuthorSearchHit {
                author_id: document.author_id,
                canonical_name: document.canonical_name.clone(),
                email: document.email.clone(),
                aliases: document.aliases.clone(),
                mailing_lists: document.mailing_lists.clone(),
                first_seen: document
                    .first_seen_ts
                    .and_then(|ts| timestamp_to_datetime(ts)),
                last_seen: document
                    .last_seen_ts
                    .and_then(|ts| timestamp_to_datetime(ts)),
                first_email_date: document
                    .first_email_ts
                    .and_then(|ts| timestamp_to_datetime(ts)),
                last_email_date: document
                    .last_email_ts
                    .and_then(|ts| timestamp_to_datetime(ts)),
                thread_count: document.thread_count,
                email_count: document.email_count,
                mailing_list_stats: document
                    .mailing_list_stats
                    .iter()
                    .map(|stats| AuthorSearchMailingListStats {
                        slug: stats.slug.clone(),
                        email_count: stats.email_count,
                        thread_count: stats.thread_count,
                        first_email_date: stats.first_email_date(),
                        last_email_date: stats.last_email_date(),
                    })
                    .collect(),
            }
        })
        .collect()
}

fn build_thread_highlights(value: &JsonValue) -> Option<ThreadSearchHighlights> {
    let subject_html = value
        .get("subject")
        .and_then(|item| item.as_str())
        .map(|s| s.to_string());
    let discussion_html = value
        .get("discussion_text")
        .and_then(|item| item.as_str())
        .map(|s| s.to_string());

    if subject_html.is_none() && discussion_html.is_none() {
        return None;
    }

    let subject_text = subject_html.as_ref().map(|html| strip_markup(html));
    let discussion_text = discussion_html.as_ref().map(|html| strip_markup(html));

    Some(ThreadSearchHighlights {
        subject_html,
        subject_text,
        discussion_html,
        discussion_text,
    })
}

fn collect_participants(document: &crate::search::ThreadDocument) -> Vec<ThreadSearchParticipant> {
    let mut participants = Vec::new();
    let limit = document
        .participant_ids
        .len()
        .min(document.participants.len())
        .min(10);

    for index in 0..limit {
        let id = document.participant_ids[index];
        let display = document.participants[index].clone();
        let email = document
            .participant_emails
            .get(index)
            .cloned()
            .unwrap_or_else(|| display.clone());
        let name = if display == email || display.is_empty() {
            None
        } else {
            Some(display)
        };

        participants.push(ThreadSearchParticipant { id, name, email });
    }

    participants
}

fn strip_markup(value: &str) -> String {
    value
        .replace("<em>", "")
        .replace("</em>", "")
        .replace("<mark>", "")
        .replace("</mark>", "")
}

fn timestamp_to_datetime(ts: i64) -> Option<DateTime<Utc>> {
    use chrono::TimeZone;
    Utc.timestamp_opt(ts, 0).single()
}

fn author_sort_descriptor(
    field: AuthorSortField,
    order: SortOrder,
) -> (SortDescriptor, Option<String>) {
    let direction = match order {
        SortOrder::Asc => SortDirection::Asc,
        SortOrder::Desc => SortDirection::Desc,
    };

    let order_str = if matches!(order, SortOrder::Asc) {
        "asc"
    } else {
        "desc"
    };

    let (api_field, meili_field) = match field {
        AuthorSortField::EmailCount => ("emailCount", Some("email_count")),
        AuthorSortField::ThreadCount => ("threadCount", Some("thread_count")),
        AuthorSortField::FirstEmailDate => ("firstEmailDate", Some("first_email_ts")),
        AuthorSortField::LastEmailDate => ("lastEmailDate", Some("last_email_ts")),
        AuthorSortField::CanonicalName => ("canonicalName", None),
        AuthorSortField::Email => ("email", None),
    };

    (
        SortDescriptor {
            field: api_field.to_string(),
            direction,
        },
        meili_field.map(|field| format!("{field}:{order_str}")),
    )
}

fn author_sort_field_name(field: AuthorSortField) -> &'static str {
    match field {
        AuthorSortField::CanonicalName => "canonicalName",
        AuthorSortField::Email => "email",
        AuthorSortField::EmailCount => "emailCount",
        AuthorSortField::ThreadCount => "threadCount",
        AuthorSortField::FirstEmailDate => "firstEmailDate",
        AuthorSortField::LastEmailDate => "lastEmailDate",
    }
}
