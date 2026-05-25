use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult, Content, ListToolsResult, Tool};
use serde_json::{Value, json};
use tracing::instrument;

use crate::config::Config;
use crate::stash::{
    StashClient,
    client::{average_rating, count_by_rating, tag_frequency},
    filters::{PerformerFilterBuilder, SceneFilterBuilder},
};

// ── Schema helpers ────────────────────────────────────────────────────────────

fn schema(properties: Value, required: &[&str]) -> Arc<rmcp::model::JsonObject> {
    Arc::new(rmcp::model::object(json!({
        "type": "object",
        "properties": properties,
        "required": required,
    })))
}

fn empty_schema() -> Arc<rmcp::model::JsonObject> {
    Arc::new(rmcp::model::object(json!({
        "type": "object",
        "properties": {},
    })))
}

// ── Tool registry ─────────────────────────────────────────────────────────────

pub fn list_tools() -> ListToolsResult {
    let mut result = ListToolsResult::default();
    result.tools = vec![
        Tool::new(
            "get_performer_info",
            "Return detailed information for a single performer by exact name",
            schema(
                json!({
                    "performer_name": {
                        "type": "string",
                        "description": "Exact name of the performer"
                    }
                }),
                &["performer_name"],
            ),
        ),
        Tool::new(
            "get_all_performers",
            "Return a list of performers with advanced filtering options",
            schema(
                json!({
                    "favorites_only": {
                        "type": "boolean",
                        "description": "Limit to favorite performers (default: true)"
                    },
                    "country": { "type": "string", "description": "Filter by country" },
                    "country_modifier": {
                        "type": "string",
                        "description": "Modifier for country filter (EQUALS|INCLUDES|NOT_EQUALS)",
                        "default": "EQUALS"
                    },
                    "ethnicity": { "type": "string", "description": "Filter by ethnicity" },
                    "ethnicity_modifier": { "type": "string", "default": "EQUALS" },
                    "eye_color": { "type": "string", "description": "Filter by eye color" },
                    "eye_color_modifier": { "type": "string", "default": "EQUALS" },
                    "hair_color": { "type": "string", "description": "Filter by hair color" },
                    "hair_color_modifier": { "type": "string", "default": "EQUALS" },
                    "height_cm": { "type": "integer", "description": "Filter by height (cm)" },
                    "height_cm_modifier": { "type": "string", "default": "EQUALS" },
                    "height_cm_value2": {
                        "type": "integer",
                        "description": "Upper bound when modifier is BETWEEN"
                    },
                    "measurements": { "type": "string", "description": "Filter by measurements" },
                    "measurements_modifier": { "type": "string", "default": "EQUALS" },
                    "piercings": { "type": "string", "description": "Filter by piercings (INCLUDES)" },
                    "tattoos": { "type": "string", "description": "Filter by tattoos (INCLUDES)" },
                    "weight": { "type": "integer", "description": "Filter by weight (kg)" },
                    "weight_modifier": { "type": "string", "default": "EQUALS" },
                    "weight_value2": {
                        "type": "integer",
                        "description": "Upper bound when modifier is BETWEEN"
                    }
                }),
                &[],
            ),
        ),
        Tool::new(
            "get_all_scenes",
            "Return all scenes from Stash with advanced filtering options",
            schema(
                json!({
                    "organized_only": {
                        "type": "boolean",
                        "description": "Only return organized scenes (default: true)"
                    },
                    "include_tags": {
                        "type": "string",
                        "description": "Comma-separated tag names that must be present"
                    },
                    "exclude_tags": {
                        "type": "string",
                        "description": "Comma-separated tag names to exclude"
                    },
                    "min_rating": {
                        "type": "integer",
                        "description": "Minimum rating 0–100 (inclusive)"
                    },
                    "max_rating": {
                        "type": "integer",
                        "description": "Maximum rating 0–100 (inclusive)"
                    }
                }),
                &[],
            ),
        ),
        Tool::new(
            "get_all_scenes_from_performer",
            "Return all scenes for a given performer",
            schema(
                json!({
                    "performer_name": {
                        "type": "string",
                        "description": "Exact name of the performer"
                    },
                    "organized_only": {
                        "type": "boolean",
                        "description": "Only return organized scenes (default: true)"
                    }
                }),
                &["performer_name"],
            ),
        ),
        Tool::new(
            "health_check",
            "Return basic health/connectivity information for the MCP server",
            empty_schema(),
        ),
        Tool::new(
            "advanced_performer_analysis",
            "Advanced performer analysis: scene stats, tag frequency, and similar performers",
            schema(
                json!({
                    "performer_name": {
                        "type": "string",
                        "description": "Name of the performer to analyze"
                    },
                    "include_similar": {
                        "type": "boolean",
                        "description": "Include similar performer suggestions (default: true)"
                    },
                    "deep_scene_analysis": {
                        "type": "boolean",
                        "description": "Perform deep rating breakdown analysis (default: false)"
                    }
                }),
                &["performer_name"],
            ),
        ),
        Tool::new(
            "batch_performer_insights",
            "Generate insights for multiple performers at once",
            schema(
                json!({
                    "performer_names": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of performer names to analyze"
                    },
                    "max_performers": {
                        "type": "integer",
                        "description": "Maximum number of performers to process (default: 10)"
                    }
                }),
                &["performer_names"],
            ),
        ),
    ];
    result
}

// ── Tool dispatch ─────────────────────────────────────────────────────────────

pub async fn call_tool(
    client: &StashClient,
    config: &Config,
    req: CallToolRequestParams,
) -> Result<CallToolResult, McpError> {
    let args = req.arguments.unwrap_or_default();

    match req.name.as_ref() {
        "get_performer_info" => get_performer_info(client, config, &args).await,
        "get_all_performers" => get_all_performers(client, config, &args).await,
        "get_all_scenes" => get_all_scenes(client, &args).await,
        "get_all_scenes_from_performer" => {
            get_all_scenes_from_performer(client, config, &args).await
        }
        "health_check" => health_check(client, config).await,
        "advanced_performer_analysis" => advanced_performer_analysis(client, config, &args).await,
        "batch_performer_insights" => batch_performer_insights(client, config, &args).await,
        other => Err(McpError::invalid_params(
            format!("unknown tool '{other}'"),
            None,
        )),
    }
}

// ── Individual tool implementations ───────────────────────────────────────────

fn arg_str<'a>(args: &'a rmcp::model::JsonObject, key: &str) -> Option<&'a str> {
    args.get(key)?.as_str()
}

fn arg_bool(args: &rmcp::model::JsonObject, key: &str, default: bool) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(default)
}

fn arg_i64(args: &rmcp::model::JsonObject, key: &str) -> Option<i64> {
    args.get(key)?.as_i64()
}

fn ok_json(value: impl serde::Serialize) -> Result<CallToolResult, McpError> {
    match serde_json::to_string_pretty(&value) {
        Ok(text) => Ok(CallToolResult::success(vec![Content::text(text)])),
        Err(e) => Err(McpError::internal_error(e.to_string(), None)),
    }
}

fn err_text(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(msg.into())])
}

fn stash_err(e: crate::error::StashError) -> CallToolResult {
    err_text(e.to_string())
}

#[instrument(skip(client, config))]
async fn get_performer_info(
    client: &StashClient,
    config: &Config,
    args: &rmcp::model::JsonObject,
) -> Result<CallToolResult, McpError> {
    let name = match arg_str(args, "performer_name") {
        Some(n) => n,
        None => return Ok(err_text("missing required argument 'performer_name'")),
    };

    match client.find_performer_by_name(name).await {
        Ok(Some(p)) => {
            let link = config.performer_url(&p.id);
            let mut v = serde_json::to_value(&p).unwrap();
            v["link"] = json!(link);
            ok_json(v)
        }
        Ok(None) => Ok(err_text(format!("performer '{name}' not found"))),
        Err(e) => Ok(stash_err(e)),
    }
}

#[instrument(skip(client, config))]
async fn get_all_performers(
    client: &StashClient,
    config: &Config,
    args: &rmcp::model::JsonObject,
) -> Result<CallToolResult, McpError> {
    let favorites_only = arg_bool(args, "favorites_only", true);

    let mut fb = PerformerFilterBuilder::new().favorites_only(favorites_only);

    if let (Some(v), mod_) = (
        arg_str(args, "country"),
        arg_str(args, "country_modifier").unwrap_or("EQUALS"),
    ) {
        fb = fb.country(v, mod_);
    }
    if let (Some(v), mod_) = (
        arg_str(args, "ethnicity"),
        arg_str(args, "ethnicity_modifier").unwrap_or("EQUALS"),
    ) {
        fb = fb.ethnicity(v, mod_);
    }
    if let (Some(v), mod_) = (
        arg_str(args, "eye_color"),
        arg_str(args, "eye_color_modifier").unwrap_or("EQUALS"),
    ) {
        fb = fb.eye_color(v, mod_);
    }
    if let (Some(v), mod_) = (
        arg_str(args, "hair_color"),
        arg_str(args, "hair_color_modifier").unwrap_or("EQUALS"),
    ) {
        fb = fb.hair_color(v, mod_);
    }
    if let Some(h) = arg_i64(args, "height_cm") {
        let mod_ = arg_str(args, "height_cm_modifier").unwrap_or("EQUALS");
        let v2 = arg_i64(args, "height_cm_value2");
        fb = fb.height_cm(h, mod_, v2);
    }
    if let (Some(v), mod_) = (
        arg_str(args, "measurements"),
        arg_str(args, "measurements_modifier").unwrap_or("EQUALS"),
    ) {
        fb = fb.measurements(v, mod_);
    }
    if let Some(v) = arg_str(args, "piercings") {
        fb = fb.piercings(v);
    }
    if let Some(v) = arg_str(args, "tattoos") {
        fb = fb.tattoos(v);
    }
    if let Some(w) = arg_i64(args, "weight") {
        let mod_ = arg_str(args, "weight_modifier").unwrap_or("EQUALS");
        let v2 = arg_i64(args, "weight_value2");
        fb = fb.weight(w, mod_, v2);
    }

    match client.find_performers(fb.build()).await {
        Ok(performers) => {
            let with_links: Vec<Value> = performers
                .iter()
                .map(|p| {
                    let mut v = serde_json::to_value(p).unwrap();
                    v["link"] = json!(config.performer_url(&p.id));
                    v
                })
                .collect();
            ok_json(with_links)
        }
        Err(e) => Ok(stash_err(e)),
    }
}

#[instrument(skip(client))]
async fn get_all_scenes(
    client: &StashClient,
    args: &rmcp::model::JsonObject,
) -> Result<CallToolResult, McpError> {
    let organized_only = arg_bool(args, "organized_only", true);
    let include_tags_str = arg_str(args, "include_tags");
    let exclude_tags_str = arg_str(args, "exclude_tags");
    let min_rating = arg_i64(args, "min_rating");
    let max_rating = arg_i64(args, "max_rating");

    let mut sf = SceneFilterBuilder::new()
        .organized_only(organized_only)
        .has_tags();

    // Resolve tag names to IDs.
    if let Some(tags_csv) = include_tags_str {
        let names: Vec<&str> = tags_csv.split(',').map(str::trim).collect();
        match client.resolve_tag_ids(&names).await {
            Ok(ids) => sf = sf.include_tags(&ids),
            Err(e) => return Ok(stash_err(e)),
        }
    } else if let Some(tags_csv) = exclude_tags_str {
        let names: Vec<&str> = tags_csv.split(',').map(str::trim).collect();
        match client.resolve_tag_ids(&names).await {
            Ok(ids) => sf = sf.exclude_tags(&ids),
            Err(e) => return Ok(stash_err(e)),
        }
    }

    sf = match (min_rating, max_rating) {
        (Some(min), Some(max)) => sf.rating_range(min, max),
        (Some(min), None) => sf.min_rating(min),
        (None, Some(max)) => sf.max_rating(max),
        (None, None) => sf,
    };

    match client.find_scenes(sf.build()).await {
        Ok(scenes) => ok_json(scenes),
        Err(e) => Ok(stash_err(e)),
    }
}

#[instrument(skip(client, config))]
async fn get_all_scenes_from_performer(
    client: &StashClient,
    config: &Config,
    args: &rmcp::model::JsonObject,
) -> Result<CallToolResult, McpError> {
    let name = match arg_str(args, "performer_name") {
        Some(n) => n,
        None => return Ok(err_text("missing required argument 'performer_name'")),
    };
    let organized_only = arg_bool(args, "organized_only", true);

    match client.find_scenes_for_performer(name, organized_only).await {
        Ok(scenes) => {
            let with_links: Vec<Value> = scenes
                .iter()
                .map(|s| {
                    let mut v = serde_json::to_value(s).unwrap();
                    v["link"] = json!(config.scene_url(&s.id));
                    v
                })
                .collect();
            ok_json(with_links)
        }
        Err(e) => Ok(stash_err(e)),
    }
}

#[instrument(skip(client, config))]
async fn health_check(client: &StashClient, config: &Config) -> Result<CallToolResult, McpError> {
    match client.health_check().await {
        Ok(ver) => ok_json(json!({
            "connected": true,
            "endpoint": config.base_url(),
            "stash_version": ver.version,
            "build_time": ver.build_time,
        })),
        Err(e) => ok_json(json!({
            "connected": false,
            "endpoint": config.base_url(),
            "error": e.to_string(),
        })),
    }
}

#[instrument(skip(client, config))]
async fn advanced_performer_analysis(
    client: &StashClient,
    config: &Config,
    args: &rmcp::model::JsonObject,
) -> Result<CallToolResult, McpError> {
    let name = match arg_str(args, "performer_name") {
        Some(n) => n,
        None => return Ok(err_text("missing required argument 'performer_name'")),
    };
    let include_similar = arg_bool(args, "include_similar", true);
    let deep = arg_bool(args, "deep_scene_analysis", false);

    // Phase 1: performer info.
    let performer = match client.find_performer_by_name(name).await {
        Ok(Some(p)) => p,
        Ok(None) => return Ok(err_text(format!("performer '{name}' not found"))),
        Err(e) => return Ok(stash_err(e)),
    };

    let mut performer_json = serde_json::to_value(&performer).unwrap();
    performer_json["link"] = json!(config.performer_url(&performer.id));

    // Phase 2: scenes.
    let scenes = client
        .find_scenes_for_performer(name, false)
        .await
        .unwrap_or_default();

    let top_rated: Vec<Value> = scenes
        .iter()
        .filter(|s| s.rating100.unwrap_or(0) > 80)
        .take(5)
        .map(|s| {
            let mut v = serde_json::to_value(s).unwrap();
            v["link"] = json!(config.scene_url(&s.id));
            v
        })
        .collect();

    let freq = tag_frequency(&scenes);
    let mut freq_sorted: Vec<(&String, &usize)> = freq.iter().collect();
    freq_sorted.sort_by(|a, b| b.1.cmp(a.1));
    let top_tags: Vec<Value> = freq_sorted
        .iter()
        .take(10)
        .map(|(name, count)| json!({ "tag": name, "count": count }))
        .collect();

    let scene_stats = json!({
        "total_scenes": scenes.len(),
        "average_rating": average_rating(&scenes),
        "top_rated_scenes": top_rated,
        "top_tags": top_tags,
    });

    // Phase 3: similar performers (optional).
    let similar: Option<Vec<Value>> = if include_similar {
        let mut fb = crate::stash::filters::PerformerFilterBuilder::new();
        if let Some(c) = &performer.country {
            fb = fb.country(c, "EQUALS");
        }
        if let Some(e) = &performer.ethnicity {
            fb = fb.ethnicity(e, "EQUALS");
        }
        let all = client.find_performers(fb.build()).await.unwrap_or_default();
        let similar_list: Vec<Value> = all
            .into_iter()
            .filter(|p| p.name.to_lowercase() != name.to_lowercase())
            .take(5)
            .map(|p| {
                let mut v = serde_json::to_value(&p).unwrap();
                v["link"] = json!(config.performer_url(&p.id));
                v
            })
            .collect();
        Some(similar_list)
    } else {
        None
    };

    // Phase 4: deep analysis (optional).
    let deep_analysis: Option<Value> = if deep {
        Some(json!({
            "scenes_by_rating": {
                "excellent_90_plus": count_by_rating(&scenes, 90, i64::MAX),
                "good_70_to_89": count_by_rating(&scenes, 70, 89),
                "average_50_to_69": count_by_rating(&scenes, 50, 69),
                "below_average_0_to_49": count_by_rating(&scenes, 0, 49),
            },
            "all_tags_sorted": top_tags,
        }))
    } else {
        None
    };

    let mut result = json!({
        "performer_info": performer_json,
        "scene_statistics": scene_stats,
        "analysis_metadata": {
            "include_similar": include_similar,
            "deep_analysis": deep,
        }
    });

    if let Some(s) = similar {
        result["similar_performers"] = json!(s);
    }
    if let Some(d) = deep_analysis {
        result["detailed_scene_analysis"] = d;
    }

    ok_json(result)
}

#[instrument(skip(client, config))]
async fn batch_performer_insights(
    client: &StashClient,
    config: &Config,
    args: &rmcp::model::JsonObject,
) -> Result<CallToolResult, McpError> {
    let names_val = args.get("performer_names").ok_or_else(|| {
        McpError::invalid_params("missing required argument 'performer_names'", None)
    })?;
    let names_arr = names_val
        .as_array()
        .ok_or_else(|| McpError::invalid_params("'performer_names' must be an array", None))?;
    let max = arg_i64(args, "max_performers").unwrap_or(10) as usize;

    let names: Vec<&str> = names_arr
        .iter()
        .filter_map(|v| v.as_str())
        .take(max)
        .collect();

    let mut processed: Vec<Value> = Vec::new();
    let mut failed: Vec<&str> = Vec::new();

    for name in &names {
        match client.find_performer_by_name(name).await {
            Ok(Some(p)) => {
                let scenes = client
                    .find_scenes_for_performer(name, false)
                    .await
                    .unwrap_or_default();
                let mut pv = serde_json::to_value(&p).unwrap();
                pv["link"] = json!(config.performer_url(&p.id));
                processed.push(json!({
                    "name": name,
                    "info": pv,
                    "scene_count": scenes.len(),
                    "average_rating": average_rating(&scenes),
                }));
            }
            Ok(None) | Err(_) => failed.push(name),
        }
    }

    let countries: Vec<String> = processed
        .iter()
        .filter_map(|p| p["info"]["country"].as_str().map(str::to_owned))
        .collect();

    let mut country_dist = serde_json::Map::new();
    for c in &countries {
        let count = country_dist
            .entry(c.clone())
            .or_insert(json!(0))
            .as_i64()
            .unwrap_or(0);
        country_dist.insert(c.clone(), json!(count + 1));
    }

    ok_json(json!({
        "summary": {
            "total_requested": names.len(),
            "total_processed": processed.len(),
            "total_failed": failed.len(),
            "average_scenes_per_performer": if processed.is_empty() { 0.0 } else {
                processed.iter()
                    .filter_map(|p| p["scene_count"].as_f64())
                    .sum::<f64>() / processed.len() as f64
            },
        },
        "demographics": {
            "country_distribution": country_dist,
        },
        "performers": processed,
        "failed_performers": failed,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_tools_returns_seven_tools() {
        let result = list_tools();
        assert_eq!(result.tools.len(), 7);
    }

    #[test]
    fn list_tools_names_are_correct() {
        let result = list_tools();
        let names: Vec<&str> = result.tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(names.contains(&"get_performer_info"));
        assert!(names.contains(&"get_all_performers"));
        assert!(names.contains(&"get_all_scenes"));
        assert!(names.contains(&"get_all_scenes_from_performer"));
        assert!(names.contains(&"health_check"));
        assert!(names.contains(&"advanced_performer_analysis"));
        assert!(names.contains(&"batch_performer_insights"));
    }

    #[test]
    fn list_tools_all_have_descriptions() {
        let result = list_tools();
        for tool in &result.tools {
            assert!(
                tool.description.is_some(),
                "tool '{}' has no description",
                tool.name
            );
        }
    }

    #[test]
    fn list_tools_all_have_schemas() {
        let result = list_tools();
        for tool in &result.tools {
            let schema = tool.schema_as_json_value();
            assert_eq!(
                schema["type"], "object",
                "tool '{}' schema root is not 'object'",
                tool.name
            );
        }
    }

    #[test]
    fn arg_str_extracts_string() {
        let mut map = rmcp::model::JsonObject::new();
        map.insert("key".to_owned(), json!("value"));
        assert_eq!(arg_str(&map, "key"), Some("value"));
        assert!(arg_str(&map, "missing").is_none());
    }

    #[test]
    fn arg_bool_uses_default_when_missing() {
        let map = rmcp::model::JsonObject::new();
        assert!(arg_bool(&map, "flag", true));
        assert!(!arg_bool(&map, "flag", false));
    }

    #[test]
    fn arg_bool_reads_value() {
        let mut map = rmcp::model::JsonObject::new();
        map.insert("flag".to_owned(), json!(false));
        assert!(!arg_bool(&map, "flag", true));
    }

    #[test]
    fn arg_i64_extracts_integer() {
        let mut map = rmcp::model::JsonObject::new();
        map.insert("n".to_owned(), json!(42));
        assert_eq!(arg_i64(&map, "n"), Some(42));
        assert!(arg_i64(&map, "missing").is_none());
    }
}
