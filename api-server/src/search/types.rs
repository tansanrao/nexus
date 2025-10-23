use rocket::form::{self, FromFormField, ValueField};
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Search execution modes supported by the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    /// Lexical-only search backed by Postgres FTS + trigram similarity.
    Lexical,
    /// Semantic-only search powered by vector similarity.
    Semantic,
    /// Hybrid search combining lexical and semantic signals.
    Hybrid,
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::Hybrid
    }
}

impl fmt::Display for SearchMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SearchMode::Lexical => "lexical",
            SearchMode::Semantic => "semantic",
            SearchMode::Hybrid => "hybrid",
        })
    }
}

impl FromStr for SearchMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "lexical" => Ok(SearchMode::Lexical),
            "semantic" => Ok(SearchMode::Semantic),
            "hybrid" | "" => Ok(SearchMode::Hybrid),
            _ => Err(()),
        }
    }
}

impl<'r> FromFormField<'r> for SearchMode {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        let value = field.value.trim();
        if value.is_empty() {
            return Ok(SearchMode::Hybrid);
        }

        Ok(SearchMode::from_str(value).map_err(|_| {
            form::Error::validation(format!("invalid search mode '{}'", field.value))
        })?)
    }
}
