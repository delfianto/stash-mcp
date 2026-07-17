use rmcp::ErrorData as McpError;
use rmcp::model::{
    ListResourceTemplatesResult, ListResourcesResult, ReadResourceRequestParams,
    ReadResourceResult, Resource, ResourceContents, ResourceTemplate,
};
use serde_json::json;
use tracing::instrument;

use crate::config::Config;
use crate::stash::{
    StashClient,
    filters::{PerformerFilterBuilder, StudioFilterBuilder, TagFilterBuilder},
};

const MIME_JSON: &str = "application/json";

fn text_resource_contents(uri: &str, text: String) -> ResourceContents {
    ResourceContents::text(text, uri).with_mime_type(MIME_JSON)
}

fn json_result(uri: &str, value: impl serde::Serialize) -> Result<ReadResourceResult, McpError> {
    let text = serde_json::to_string_pretty(&value)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    Ok(ReadResourceResult::new(vec![text_resource_contents(
        uri, text,
    )]))
}

fn err_result(uri: &str, message: impl Into<String>) -> Result<ReadResourceResult, McpError> {
    let text = serde_json::to_string_pretty(&json!({
        "success": false,
        "error": message.into()
    }))
    .unwrap();
    Ok(ReadResourceResult::new(vec![text_resource_contents(
        uri, text,
    )]))
}

// ── Static resource list ──────────────────────────────────────────────────────

pub fn list_resources() -> ListResourcesResult {
    let static_uris = [
        (
            "stash://performer/all",
            "All performers",
            "List of all favorite performers in Stash",
        ),
        (
            "stash://performer/stats",
            "Performers Statistics",
            "Statistical summary of all performers",
        ),
        (
            "stash://studio/all",
            "All studios",
            "List of all favorite studios in Stash",
        ),
        (
            "stash://studio/stats",
            "Studios Statistics",
            "Statistical summary of all studios",
        ),
        (
            "stash://tag/all",
            "All tags",
            "List of all favorite tags in Stash",
        ),
        (
            "stash://tag/stats",
            "Tags Statistics",
            "Statistical summary of all tags",
        ),
    ];

    let resources = static_uris
        .iter()
        .map(|(uri, name, desc)| {
            Resource::new(*uri, *name)
                .with_description(*desc)
                .with_mime_type(MIME_JSON)
        })
        .collect();

    let mut result = ListResourcesResult::default();
    result.resources = resources;
    result
}

pub fn list_resource_templates() -> ListResourceTemplatesResult {
    let templates = [
        (
            "stash://performer/{name}",
            "Performer Information",
            "Detailed information about a specific performer",
        ),
        (
            "stash://performer/country/{country}",
            "Performers by Country",
            "List of performers from a specific country",
        ),
        (
            "stash://performer/ethnicity/{ethnicity}",
            "Performers by Ethnicity",
            "List of performers with a specific ethnicity",
        ),
        (
            "stash://studio/{name}",
            "Studio Information",
            "Detailed information about a specific studio",
        ),
        (
            "stash://tag/{name}",
            "Tag Information",
            "Detailed information about a specific tag",
        ),
    ];

    let resource_templates = templates
        .iter()
        .map(|(tpl, name, desc)| {
            ResourceTemplate::new(*tpl, *name)
                .with_description(*desc)
                .with_mime_type(MIME_JSON)
        })
        .collect();

    let mut result = ListResourceTemplatesResult::default();
    result.resource_templates = resource_templates;
    result
}

// ── Resource router ───────────────────────────────────────────────────────────

#[instrument(skip(client, config))]
pub async fn read_resource(
    client: &StashClient,
    config: &Config,
    req: ReadResourceRequestParams,
) -> Result<ReadResourceResult, McpError> {
    let uri = req.uri.as_str();

    // Static resources.
    if uri == "stash://performer/all" {
        return resource_all_performers(client, config, uri).await;
    }
    if uri == "stash://performer/stats" {
        return resource_performer_stats(client, uri).await;
    }
    if uri == "stash://studio/all" {
        return resource_all_studios(client, config, uri).await;
    }
    if uri == "stash://studio/stats" {
        return resource_studio_stats(client, uri).await;
    }
    if uri == "stash://tag/all" {
        return resource_all_tags(client, uri).await;
    }
    if uri == "stash://tag/stats" {
        return resource_tag_stats(client, uri).await;
    }

    // Template resources.
    if let Some(name) = uri.strip_prefix("stash://performer/country/") {
        return resource_performers_by_country(client, config, uri, name).await;
    }
    if let Some(ethnicity) = uri.strip_prefix("stash://performer/ethnicity/") {
        return resource_performers_by_ethnicity(client, config, uri, ethnicity).await;
    }
    if let Some(name) = uri.strip_prefix("stash://performer/") {
        return resource_performer_by_name(client, uri, name).await;
    }
    if let Some(name) = uri.strip_prefix("stash://studio/") {
        return resource_studio_by_name(client, uri, name).await;
    }
    if let Some(name) = uri.strip_prefix("stash://tag/") {
        return resource_tag_by_name(client, uri, name).await;
    }

    Err(McpError::invalid_params(
        format!("unknown resource URI '{uri}'"),
        None,
    ))
}

// ── Performer resources ───────────────────────────────────────────────────────

async fn resource_all_performers(
    client: &StashClient,
    config: &Config,
    uri: &str,
) -> Result<ReadResourceResult, McpError> {
    let pf = PerformerFilterBuilder::new()
        .favorites_only(config.favorites_only)
        .build();
    match client.find_performers(pf).await {
        Ok(performers) => {
            let list: Vec<_> = performers
                .iter()
                .map(|p| {
                    json!({
                        "name": p.name,
                        "country": p.country,
                        "ethnicity": p.ethnicity,
                        "height_cm": p.height_cm,
                        "weight": p.weight,
                        "tags": p.tags.as_ref().map(|ts| ts.iter().map(|t| &t.name).collect::<Vec<_>>()),
                    })
                })
                .collect();
            json_result(
                uri,
                json!({ "success": true, "total": list.len(), "performers": list }),
            )
        }
        Err(e) => err_result(uri, e.to_string()),
    }
}

async fn resource_performer_by_name(
    client: &StashClient,
    uri: &str,
    name: &str,
) -> Result<ReadResourceResult, McpError> {
    match client.find_performer_by_name(name).await {
        Ok(Some(p)) => json_result(uri, json!({ "success": true, "performer": p })),
        Ok(None) => err_result(uri, format!("performer '{name}' not found")),
        Err(e) => err_result(uri, e.to_string()),
    }
}

async fn resource_performers_by_country(
    client: &StashClient,
    config: &Config,
    uri: &str,
    country: &str,
) -> Result<ReadResourceResult, McpError> {
    let pf = PerformerFilterBuilder::new()
        .favorites_only(config.favorites_only)
        .country(country, "EQUALS")
        .build();
    match client.find_performers(pf).await {
        Ok(performers) => {
            let list: Vec<_> = performers
                .iter()
                .map(|p| json!({ "name": p.name, "ethnicity": p.ethnicity }))
                .collect();
            json_result(
                uri,
                json!({ "success": true, "country": country, "total": list.len(), "performers": list }),
            )
        }
        Err(e) => err_result(uri, e.to_string()),
    }
}

async fn resource_performers_by_ethnicity(
    client: &StashClient,
    config: &Config,
    uri: &str,
    ethnicity: &str,
) -> Result<ReadResourceResult, McpError> {
    let pf = PerformerFilterBuilder::new()
        .favorites_only(config.favorites_only)
        .ethnicity(ethnicity, "EQUALS")
        .build();
    match client.find_performers(pf).await {
        Ok(performers) => {
            let list: Vec<_> = performers
                .iter()
                .map(|p| json!({ "name": p.name, "country": p.country }))
                .collect();
            json_result(
                uri,
                json!({ "success": true, "ethnicity": ethnicity, "total": list.len(), "performers": list }),
            )
        }
        Err(e) => err_result(uri, e.to_string()),
    }
}

async fn resource_performer_stats(
    client: &StashClient,
    uri: &str,
) -> Result<ReadResourceResult, McpError> {
    let pf = PerformerFilterBuilder::new().favorites_only(true).build();
    match client.find_performers(pf).await {
        Ok(performers) => {
            let mut countries: std::collections::HashMap<String, usize> = Default::default();
            let mut ethnicities: std::collections::HashMap<String, usize> = Default::default();
            let mut heights: Vec<i64> = vec![];
            let mut weights: Vec<i64> = vec![];

            for p in &performers {
                if let Some(c) = &p.country {
                    *countries.entry(c.clone()).or_insert(0) += 1;
                }
                if let Some(e) = &p.ethnicity {
                    *ethnicities.entry(e.clone()).or_insert(0) += 1;
                }
                if let Some(h) = p.height_cm {
                    heights.push(h);
                }
                if let Some(w) = p.weight {
                    weights.push(w);
                }
            }

            let height_stats = if heights.is_empty() {
                None
            } else {
                Some(json!({
                    "average_cm": heights.iter().sum::<i64>() as f64 / heights.len() as f64,
                    "min_cm": heights.iter().min(),
                    "max_cm": heights.iter().max(),
                    "count": heights.len(),
                }))
            };

            let weight_stats = if weights.is_empty() {
                None
            } else {
                Some(json!({
                    "average_kg": weights.iter().sum::<i64>() as f64 / weights.len() as f64,
                    "min_kg": weights.iter().min(),
                    "max_kg": weights.iter().max(),
                    "count": weights.len(),
                }))
            };

            json_result(
                uri,
                json!({
                    "success": true,
                    "total_performers": performers.len(),
                    "statistics": {
                        "country_distribution": countries,
                        "ethnicity_distribution": ethnicities,
                        "height": height_stats,
                        "weight": weight_stats,
                    }
                }),
            )
        }
        Err(e) => err_result(uri, e.to_string()),
    }
}

// ── Studio resources ──────────────────────────────────────────────────────────

async fn resource_all_studios(
    client: &StashClient,
    config: &Config,
    uri: &str,
) -> Result<ReadResourceResult, McpError> {
    let sf = StudioFilterBuilder::new()
        .favorites_only(config.favorites_only)
        .build();
    match client.find_studios(sf).await {
        Ok(studios) => {
            let list: Vec<_> = studios
                .iter()
                .map(|s| {
                    json!({
                        "name": s.name,
                        "scene_count": s.scene_count,
                        "url": s.url,
                        "rating100": s.rating100,
                        "parent_studio": s.parent_studio.as_ref().map(|p| &p.name),
                    })
                })
                .collect();
            json_result(
                uri,
                json!({ "success": true, "total": list.len(), "studios": list }),
            )
        }
        Err(e) => err_result(uri, e.to_string()),
    }
}

async fn resource_studio_by_name(
    client: &StashClient,
    uri: &str,
    name: &str,
) -> Result<ReadResourceResult, McpError> {
    match client.find_studio_by_name(name).await {
        Ok(Some(s)) => json_result(uri, json!({ "success": true, "studio": s })),
        Ok(None) => err_result(uri, format!("studio '{name}' not found")),
        Err(e) => err_result(uri, e.to_string()),
    }
}

async fn resource_studio_stats(
    client: &StashClient,
    uri: &str,
) -> Result<ReadResourceResult, McpError> {
    let sf = StudioFilterBuilder::new().favorites_only(true).build();
    match client.find_studios(sf).await {
        Ok(studios) => {
            let total_scenes: i64 = studios.iter().filter_map(|s| s.scene_count).sum();
            let ratings: Vec<i64> = studios.iter().filter_map(|s| s.rating100).collect();
            let with_parent = studios.iter().filter(|s| s.parent_studio.is_some()).count();
            let with_children = studios
                .iter()
                .filter(|s| s.child_studios.as_ref().is_some_and(|c| !c.is_empty()))
                .count();

            let rating_stats = if ratings.is_empty() {
                None
            } else {
                Some(json!({
                    "average": ratings.iter().sum::<i64>() as f64 / ratings.len() as f64,
                    "min": ratings.iter().min(),
                    "max": ratings.iter().max(),
                    "count": ratings.len(),
                }))
            };

            json_result(
                uri,
                json!({
                    "success": true,
                    "total_studios": studios.len(),
                    "statistics": {
                        "total_scenes": total_scenes,
                        "hierarchy": {
                            "studios_with_parent": with_parent,
                            "studios_with_children": with_children,
                        },
                        "ratings": rating_stats,
                    }
                }),
            )
        }
        Err(e) => err_result(uri, e.to_string()),
    }
}

// ── Tag resources ─────────────────────────────────────────────────────────────

async fn resource_all_tags(
    client: &StashClient,
    uri: &str,
) -> Result<ReadResourceResult, McpError> {
    let tf = TagFilterBuilder::new().favorites_only(true).build();
    match client.find_tags(tf).await {
        Ok(tags) => {
            let list: Vec<_> = tags
                .iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "scene_count": t.scene_count,
                        "description": t.description,
                        "scene_marker_count": t.scene_marker_count,
                        "parents": t.parents.as_ref().map(|ps| ps.iter().map(|p| &p.name).collect::<Vec<_>>()),
                    })
                })
                .collect();
            json_result(
                uri,
                json!({ "success": true, "total": list.len(), "tags": list }),
            )
        }
        Err(e) => err_result(uri, e.to_string()),
    }
}

async fn resource_tag_by_name(
    client: &StashClient,
    uri: &str,
    name: &str,
) -> Result<ReadResourceResult, McpError> {
    match client.find_tag_by_name(name).await {
        Ok(Some(t)) => json_result(uri, json!({ "success": true, "tag": t })),
        Ok(None) => err_result(uri, format!("tag '{name}' not found")),
        Err(e) => err_result(uri, e.to_string()),
    }
}

async fn resource_tag_stats(
    client: &StashClient,
    uri: &str,
) -> Result<ReadResourceResult, McpError> {
    let tf = TagFilterBuilder::new().favorites_only(true).build();
    match client.find_tags(tf).await {
        Ok(tags) => {
            let total_scenes: i64 = tags.iter().filter_map(|t| t.scene_count).sum();
            let total_markers: i64 = tags.iter().filter_map(|t| t.scene_marker_count).sum();
            let with_parents = tags
                .iter()
                .filter(|t| t.parents.as_ref().is_some_and(|p| !p.is_empty()))
                .count();
            let with_children = tags
                .iter()
                .filter(|t| t.children.as_ref().is_some_and(|c| !c.is_empty()))
                .count();

            json_result(
                uri,
                json!({
                    "success": true,
                    "total_tags": tags.len(),
                    "statistics": {
                        "total_scene_associations": total_scenes,
                        "total_marker_associations": total_markers,
                        "hierarchy": {
                            "tags_with_parents": with_parents,
                            "tags_with_children": with_children,
                        }
                    }
                }),
            )
        }
        Err(e) => err_result(uri, e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_resources_returns_six_static() {
        let result = list_resources();
        assert_eq!(result.resources.len(), 6);
    }

    #[test]
    fn list_resources_uris_are_correct() {
        let result = list_resources();
        let uris: Vec<&str> = result.resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&"stash://performer/all"));
        assert!(uris.contains(&"stash://performer/stats"));
        assert!(uris.contains(&"stash://studio/all"));
        assert!(uris.contains(&"stash://studio/stats"));
        assert!(uris.contains(&"stash://tag/all"));
        assert!(uris.contains(&"stash://tag/stats"));
    }

    #[test]
    fn list_resources_all_have_json_mime() {
        let result = list_resources();
        for r in &result.resources {
            assert_eq!(
                r.mime_type.as_deref(),
                Some(MIME_JSON),
                "resource '{}' has wrong MIME",
                r.uri
            );
        }
    }

    #[test]
    fn list_resource_templates_returns_five() {
        let result = list_resource_templates();
        assert_eq!(result.resource_templates.len(), 5);
    }

    #[test]
    fn list_resource_templates_contain_placeholders() {
        let result = list_resource_templates();
        for t in &result.resource_templates {
            assert!(
                t.uri_template.contains('{'),
                "template '{}' has no placeholder",
                t.uri_template
            );
        }
    }

    #[test]
    fn text_resource_contents_sets_mime() {
        let rc = text_resource_contents("stash://test", "{}".to_owned());
        if let ResourceContents::TextResourceContents {
            uri,
            mime_type,
            text,
            ..
        } = rc
        {
            assert_eq!(uri, "stash://test");
            assert_eq!(mime_type.as_deref(), Some(MIME_JSON));
            assert_eq!(text, "{}");
        } else {
            panic!("expected text resource contents");
        }
    }
}
