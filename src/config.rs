use std::{collections::HashMap, path::Path};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub endpoint: String,
    pub api_key: String,
    #[allow(dead_code)]
    pub api_key_db: String,
    pub favorites_only: bool,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error reading config file: {0}")]
    Io(#[from] std::io::Error),
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_map(parse_env_content(&content))
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_map(std::env::vars().collect())
    }

    pub fn from_map(map: HashMap<String, String>) -> Result<Self, ConfigError> {
        let get = |key: &str, default: &str| -> String {
            map.get(key)
                .filter(|v| !v.is_empty())
                .cloned()
                .unwrap_or_else(|| default.to_owned())
        };

        Ok(Config {
            endpoint: get("STASH_HOST", "http://localhost:9999"),
            api_key: map.get("STASH_API_KEY").cloned().unwrap_or_default(),
            api_key_db: map.get("STASH_DB_API_KEY").cloned().unwrap_or_default(),
            favorites_only: get("FAVORITES_ONLY", "true").to_lowercase() != "false",
        })
    }

    pub fn base_url(&self) -> String {
        self.endpoint.trim_end_matches('/').to_string()
    }

    pub fn graphql_url(&self) -> String {
        format!("{}/graphql", self.base_url())
    }

    pub fn performer_url(&self, id: &str) -> String {
        format!("{}/performers/{}", self.base_url(), id)
    }

    pub fn scene_url(&self, id: &str) -> String {
        format!("{}/scenes/{}", self.base_url(), id)
    }
}

/// Parse a `KEY=VALUE` env-format file.
/// Skips blank lines and lines starting with `#`.
/// Strips surrounding single or double quotes from values.
/// Splits only on the first `=`, so values may contain `=`.
pub fn parse_env_content(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            if key.is_empty() {
                continue;
            }
            let value = strip_quotes(value.trim());
            map.insert(key.to_owned(), value.to_owned());
        }
    }
    map
}

fn strip_quotes(s: &str) -> &str {
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_env_content ──────────────────────────────────────────────────

    #[test]
    fn parses_basic_key_value_pairs() {
        let map = parse_env_content("STASH_SCHEME=http\nSTASH_HOST=localhost\nSTASH_PORT=9999");
        assert_eq!(map["STASH_SCHEME"], "http");
        assert_eq!(map["STASH_HOST"], "localhost");
        assert_eq!(map["STASH_PORT"], "9999");
    }

    #[test]
    fn skips_comment_lines() {
        let map = parse_env_content("# comment\nSTASH_HOST=localhost");
        assert!(!map.contains_key("# comment"));
        assert!(map.contains_key("STASH_HOST"));
    }

    #[test]
    fn skips_blank_lines() {
        let map = parse_env_content("\n\nSTASH_HOST=localhost\n\n");
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn strips_double_quotes_from_value() {
        let map = parse_env_content(r#"STASH_API_KEY="abc123""#);
        assert_eq!(map["STASH_API_KEY"], "abc123");
    }

    #[test]
    fn strips_single_quotes_from_value() {
        let map = parse_env_content("STASH_API_KEY='abc123'");
        assert_eq!(map["STASH_API_KEY"], "abc123");
    }

    #[test]
    fn preserves_empty_value() {
        let map = parse_env_content("STASH_API_KEY=");
        assert_eq!(map["STASH_API_KEY"], "");
    }

    #[test]
    fn value_containing_equals_sign() {
        let map = parse_env_content("KEY=a=b=c");
        assert_eq!(map["KEY"], "a=b=c");
    }

    #[test]
    fn trims_whitespace_around_key_and_value() {
        let map = parse_env_content("  STASH_HOST  =  localhost  ");
        assert_eq!(map["STASH_HOST"], "localhost");
    }

    // ── Config::from_map ───────────────────────────────────────────────────

    #[test]
    fn config_uses_defaults_when_map_is_empty() {
        let cfg = Config::from_map(HashMap::new()).unwrap();
        assert_eq!(cfg.endpoint, "http://localhost:9999");
        assert_eq!(cfg.api_key, "");
        assert!(cfg.favorites_only);
    }

    #[test]
    fn config_reads_keys() {
        let map: HashMap<_, _> = [
            ("STASH_HOST", "https://stash.local"),
            ("STASH_API_KEY", "secret"),
            ("STASH_DB_API_KEY", "dbsecret"),
            ("FAVORITES_ONLY", "false"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect();

        let cfg = Config::from_map(map).unwrap();
        assert_eq!(cfg.endpoint, "https://stash.local");
        assert_eq!(cfg.api_key, "secret");
        assert_eq!(cfg.api_key_db, "dbsecret");
        assert!(!cfg.favorites_only);
    }

    #[test]
    fn config_favorites_only_case_insensitive_false() {
        for value in &["false", "FALSE", "False"] {
            let map = [("FAVORITES_ONLY".to_owned(), (*value).to_owned())]
                .into_iter()
                .collect();
            assert!(!Config::from_map(map).unwrap().favorites_only, "{value}");
        }
    }

    #[test]
    fn config_favorites_only_true_by_default_and_any_other_value() {
        for value in &["true", "TRUE", "yes", "1"] {
            let map = [("FAVORITES_ONLY".to_owned(), (*value).to_owned())]
                .into_iter()
                .collect();
            assert!(Config::from_map(map).unwrap().favorites_only, "{value}");
        }
    }

    // ── URL helpers ────────────────────────────────────────────────────────

    #[test]
    fn base_url_returns_endpoint() {
        let cfg = Config::from_map(HashMap::new()).unwrap();
        assert_eq!(cfg.base_url(), "http://localhost:9999");
    }

    #[test]
    fn base_url_strips_trailing_slash() {
        let map = [("STASH_HOST".to_owned(), "https://stash.local/".to_owned())]
            .into_iter()
            .collect();
        let cfg = Config::from_map(map).unwrap();
        assert_eq!(cfg.base_url(), "https://stash.local");
    }

    #[test]
    fn graphql_url_appends_path() {
        let cfg = Config::from_map(HashMap::new()).unwrap();
        assert_eq!(cfg.graphql_url(), "http://localhost:9999/graphql");
    }

    #[test]
    fn performer_url_appends_id() {
        let cfg = Config::from_map(HashMap::new()).unwrap();
        assert_eq!(
            cfg.performer_url("42"),
            "http://localhost:9999/performers/42"
        );
    }

    #[test]
    fn scene_url_appends_id() {
        let cfg = Config::from_map(HashMap::new()).unwrap();
        assert_eq!(cfg.scene_url("7"), "http://localhost:9999/scenes/7");
    }

    // ── strip_quotes ───────────────────────────────────────────────────────

    #[test]
    fn strip_quotes_removes_double_quotes() {
        assert_eq!(strip_quotes("\"hello\""), "hello");
    }

    #[test]
    fn strip_quotes_removes_single_quotes() {
        assert_eq!(strip_quotes("'hello'"), "hello");
    }

    #[test]
    fn strip_quotes_leaves_unquoted_unchanged() {
        assert_eq!(strip_quotes("hello"), "hello");
    }

    #[test]
    fn strip_quotes_mismatched_quotes_unchanged() {
        assert_eq!(strip_quotes("\"hello'"), "\"hello'");
    }

    #[test]
    fn strip_quotes_single_char_unchanged() {
        assert_eq!(strip_quotes("\""), "\"");
    }
}
