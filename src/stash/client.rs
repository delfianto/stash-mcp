use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tracing::{debug, warn};

use crate::config::Config;
use crate::error::StashError;

use super::{
    filters::{PerformerFilterBuilder, SceneFilterBuilder, TagFilterBuilder, all_pages},
    queries,
    types::{Performer, Scene, StashVersion, Studio, Tag},
};

// ── Wire response wrappers (concrete types per entity avoid serde T:Default) ─

#[derive(Deserialize)]
struct VersionWrap {
    version: StashVersion,
}

#[derive(Deserialize)]
struct FindPerformersWrap {
    #[serde(rename = "findPerformers")]
    find_performers: PerformersResult,
}

#[derive(Deserialize)]
struct PerformersResult {
    #[allow(dead_code)]
    count: i64,
    #[serde(default)]
    performers: Vec<Performer>,
}

#[derive(Deserialize)]
struct FindScenesWrap {
    #[serde(rename = "findScenes")]
    find_scenes: ScenesResult,
}

#[derive(Deserialize)]
struct ScenesResult {
    #[allow(dead_code)]
    count: i64,
    #[serde(default)]
    scenes: Vec<Scene>,
}

#[derive(Deserialize)]
struct FindStudiosWrap {
    #[serde(rename = "findStudios")]
    find_studios: StudiosResult,
}

#[derive(Deserialize)]
struct StudiosResult {
    #[allow(dead_code)]
    count: i64,
    #[serde(default)]
    studios: Vec<Studio>,
}

#[derive(Deserialize)]
struct FindTagsWrap {
    #[serde(rename = "findTags")]
    find_tags: TagsResult,
}

#[derive(Deserialize)]
struct TagsResult {
    #[allow(dead_code)]
    count: i64,
    #[serde(default)]
    tags: Vec<Tag>,
}

// ── StashClient ───────────────────────────────────────────────────────────────

/// Thin async HTTP client that speaks directly to Stash's GraphQL endpoint.
/// All query methods return `Result<_, StashError>`.
pub struct StashClient {
    http: reqwest::Client,
    graphql_url: String,
    api_key: String,
}

impl StashClient {
    pub fn new(config: &Config) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");
        Self {
            http,
            graphql_url: config.graphql_url(),
            api_key: config.api_key.clone(),
        }
    }

    /// Send a raw GraphQL query and deserialize the `data` field into `T`.
    async fn graphql<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: Value,
    ) -> Result<T, StashError> {
        let body = json!({ "query": query, "variables": variables });
        debug!(url = %self.graphql_url, "sending GraphQL request");

        let mut req = self.http.post(&self.graphql_url).json(&body);
        if !self.api_key.is_empty() {
            req = req.header("ApiKey", &self.api_key);
        }

        let response = req.send().await?;
        let payload: Value = response.json().await?;

        // Surface any GraphQL-level errors.
        if let Some(errors) = payload.get("errors")
            && let Some(arr) = errors.as_array()
            && !arr.is_empty()
        {
            let msgs: Vec<String> = arr
                .iter()
                .filter_map(|e| e["message"].as_str().map(str::to_owned))
                .collect();
            warn!(?msgs, "GraphQL errors");
            return Err(StashError::graphql(msgs));
        }

        let data = payload
            .get("data")
            .ok_or(StashError::MissingField("data"))?;

        Ok(serde_json::from_value(data.clone())?)
    }

    // ── Health ────────────────────────────────────────────────────────────

    /// Verify connectivity: returns the Stash server version string.
    pub async fn health_check(&self) -> Result<StashVersion, StashError> {
        let wrap: VersionWrap = self.graphql(queries::HEALTH_CHECK, json!({})).await?;
        Ok(wrap.version)
    }

    // ── Performers ────────────────────────────────────────────────────────

    /// Find a single performer by exact name. Returns `None` if not found.
    pub async fn find_performer_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Performer>, StashError> {
        let pf = PerformerFilterBuilder::new().name_equals(name).build();
        let vars = json!({ "filter": all_pages(), "performer_filter": pf });
        let wrap: FindPerformersWrap = self.graphql(queries::FIND_PERFORMERS, vars).await?;
        Ok(wrap.find_performers.performers.into_iter().next())
    }

    /// List performers with an optional filter. Pass `Value::Null` for no filter.
    pub async fn find_performers(
        &self,
        performer_filter: Value,
    ) -> Result<Vec<Performer>, StashError> {
        let vars = json!({
            "filter": all_pages(),
            "performer_filter": performer_filter,
        });
        let wrap: FindPerformersWrap = self.graphql(queries::FIND_PERFORMERS, vars).await?;
        Ok(wrap.find_performers.performers)
    }

    // ── Scenes ────────────────────────────────────────────────────────────

    /// List scenes with an optional filter. Pass `Value::Null` for no filter.
    pub async fn find_scenes(&self, scene_filter: Value) -> Result<Vec<Scene>, StashError> {
        let vars = json!({
            "filter": all_pages(),
            "scene_filter": scene_filter,
        });
        let wrap: FindScenesWrap = self.graphql(queries::FIND_SCENES, vars).await?;
        Ok(wrap.find_scenes.scenes)
    }

    /// All scenes for a given performer. Resolves performer ID first.
    pub async fn find_scenes_for_performer(
        &self,
        performer_name: &str,
        organized_only: bool,
    ) -> Result<Vec<Scene>, StashError> {
        let performer = self
            .find_performer_by_name(performer_name)
            .await?
            .ok_or_else(|| StashError::not_found(format!("performer '{performer_name}'")))?;

        let sf = SceneFilterBuilder::new()
            .organized_only(organized_only)
            .performer_id(&performer.id)
            .build();

        self.find_scenes(sf).await
    }

    // ── Studios ───────────────────────────────────────────────────────────

    /// List studios with an optional filter.
    pub async fn find_studios(&self, studio_filter: Value) -> Result<Vec<Studio>, StashError> {
        let vars = json!({
            "filter": all_pages(),
            "studio_filter": studio_filter,
        });
        let wrap: FindStudiosWrap = self.graphql(queries::FIND_STUDIOS, vars).await?;
        Ok(wrap.find_studios.studios)
    }

    /// Find a single studio by exact name.
    pub async fn find_studio_by_name(&self, name: &str) -> Result<Option<Studio>, StashError> {
        // Stash's StudioFilterType doesn't have a direct name criterion in the same
        // form as performers — use text search in FindFilterType instead.
        let vars = json!({
            "filter": { "per_page": -1, "q": name },
            "studio_filter": {},
        });
        let wrap: FindStudiosWrap = self.graphql(queries::FIND_STUDIOS, vars).await?;
        // Text search may return partial matches; find the exact one.
        Ok(wrap
            .find_studios
            .studios
            .into_iter()
            .find(|s| s.name.to_lowercase() == name.to_lowercase()))
    }

    // ── Tags ──────────────────────────────────────────────────────────────

    /// List tags with an optional filter.
    pub async fn find_tags(&self, tag_filter: Value) -> Result<Vec<Tag>, StashError> {
        let vars = json!({
            "filter": all_pages(),
            "tag_filter": tag_filter,
        });
        let wrap: FindTagsWrap = self.graphql(queries::FIND_TAGS, vars).await?;
        Ok(wrap.find_tags.tags)
    }

    /// Find a single tag by exact name.
    pub async fn find_tag_by_name(&self, name: &str) -> Result<Option<Tag>, StashError> {
        let tf = TagFilterBuilder::new().name_equals(name).build();
        let tags = self.find_tags(tf).await?;
        Ok(tags
            .into_iter()
            .find(|t| t.name.to_lowercase() == name.to_lowercase()))
    }

    /// Resolve a slice of tag names to their IDs.
    /// Returns an error listing any names that could not be resolved.
    pub async fn resolve_tag_ids(&self, names: &[&str]) -> Result<Vec<String>, StashError> {
        let mut ids = Vec::with_capacity(names.len());
        let mut missing = Vec::new();

        for &name in names {
            match self.find_tag_by_name(name).await? {
                Some(tag) => ids.push(tag.id),
                None => missing.push(name.to_owned()),
            }
        }

        if !missing.is_empty() {
            return Err(StashError::graphql(vec![format!(
                "unknown tags: {}",
                missing.join(", ")
            )]));
        }

        Ok(ids)
    }
}

// ── Helpers for callers ───────────────────────────────────────────────────────

/// Compute the average of all non-null `rating100` values. Returns `None` if
/// no ratings are present.
pub fn average_rating(scenes: &[Scene]) -> Option<f64> {
    let ratings: Vec<i64> = scenes.iter().filter_map(|s| s.rating100).collect();
    if ratings.is_empty() {
        None
    } else {
        Some(ratings.iter().sum::<i64>() as f64 / ratings.len() as f64)
    }
}

/// Count how many scenes have a rating in the inclusive range `[min, max]`.
/// Pass `i64::MAX` for `max` to mean "no upper bound".
pub fn count_by_rating(scenes: &[Scene], min: i64, max: i64) -> usize {
    scenes
        .iter()
        .filter(|s| s.rating100.is_some_and(|r| r >= min && r <= max))
        .count()
}

/// Build a tag-name → occurrence-count frequency map from a scene list.
pub fn tag_frequency(scenes: &[Scene]) -> std::collections::HashMap<String, usize> {
    let mut freq = std::collections::HashMap::new();
    for scene in scenes {
        for tag in scene.tags.iter().flatten() {
            *freq.entry(tag.name.clone()).or_insert(0) += 1;
        }
    }
    freq
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stash::types::{Scene, TagRef};

    fn make_scene(rating: Option<i64>, tag_names: &[&str]) -> Scene {
        Scene {
            id: "1".to_owned(),
            rating100: rating,
            tags: if tag_names.is_empty() {
                None
            } else {
                Some(
                    tag_names
                        .iter()
                        .map(|n| TagRef {
                            id: "0".to_owned(),
                            name: n.to_string(),
                        })
                        .collect(),
                )
            },
            ..Default::default()
        }
    }

    // ── average_rating ────────────────────────────────────────────────────

    #[test]
    fn average_rating_no_scenes() {
        assert!(average_rating(&[]).is_none());
    }

    #[test]
    fn average_rating_all_null() {
        let scenes = vec![make_scene(None, &[]), make_scene(None, &[])];
        assert!(average_rating(&scenes).is_none());
    }

    #[test]
    fn average_rating_mixed() {
        let scenes = vec![
            make_scene(Some(80), &[]),
            make_scene(None, &[]),
            make_scene(Some(100), &[]),
        ];
        let avg = average_rating(&scenes).unwrap();
        assert!((avg - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn average_rating_single() {
        let scenes = vec![make_scene(Some(75), &[])];
        assert!((average_rating(&scenes).unwrap() - 75.0).abs() < f64::EPSILON);
    }

    // ── count_by_rating ───────────────────────────────────────────────────

    #[test]
    fn count_by_rating_inclusive_range() {
        let scenes = vec![
            make_scene(Some(70), &[]),
            make_scene(Some(80), &[]),
            make_scene(Some(90), &[]),
            make_scene(Some(50), &[]),
        ];
        assert_eq!(count_by_rating(&scenes, 70, 90), 3);
    }

    #[test]
    fn count_by_rating_no_match() {
        let scenes = vec![make_scene(Some(40), &[]), make_scene(None, &[])];
        assert_eq!(count_by_rating(&scenes, 70, 100), 0);
    }

    #[test]
    fn count_by_rating_null_ratings_excluded() {
        let scenes = vec![make_scene(None, &[]), make_scene(None, &[])];
        assert_eq!(count_by_rating(&scenes, 0, 100), 0);
    }

    // ── tag_frequency ─────────────────────────────────────────────────────

    #[test]
    fn tag_frequency_empty() {
        assert!(tag_frequency(&[]).is_empty());
    }

    #[test]
    fn tag_frequency_counts_correctly() {
        let scenes = vec![
            make_scene(None, &["comedy", "drama"]),
            make_scene(None, &["comedy"]),
            make_scene(None, &["action"]),
        ];
        let freq = tag_frequency(&scenes);
        assert_eq!(freq["comedy"], 2);
        assert_eq!(freq["drama"], 1);
        assert_eq!(freq["action"], 1);
    }

    #[test]
    fn tag_frequency_scene_with_no_tags() {
        let scenes = vec![make_scene(None, &[]), make_scene(None, &["x"])];
        let freq = tag_frequency(&scenes);
        assert_eq!(freq["x"], 1);
        assert_eq!(freq.len(), 1);
    }

    // ── CountedList deserialization ───────────────────────────────────────

    #[test]
    fn find_performers_wrap_deserializes() {
        let json = r#"{
            "findPerformers": {
                "count": 1,
                "performers": [{"id": "1", "name": "Alice"}]
            }
        }"#;
        let wrap: FindPerformersWrap = serde_json::from_str(json).unwrap();
        assert_eq!(wrap.find_performers.performers[0].name, "Alice");
    }

    #[test]
    fn find_scenes_wrap_deserializes() {
        let json = r#"{
            "findScenes": {
                "count": 1,
                "scenes": [{"id": "5", "title": "Test"}]
            }
        }"#;
        let wrap: FindScenesWrap = serde_json::from_str(json).unwrap();
        assert_eq!(wrap.find_scenes.scenes[0].id, "5");
    }
}
