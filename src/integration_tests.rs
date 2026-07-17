/// Deep integration tests against a live Stash instance.
///
/// These tests are skipped automatically when no `.env` file is present in the
/// working directory.  Run them with:
///
///   cargo test integration -- --nocapture
///
/// All tests hit the real GraphQL endpoint configured in `.env`, so they
/// exercise the full stack: config loading → HTTP → deserialization → tool
/// dispatch.
#[cfg(test)]
mod tests {
    use std::path::Path;

    use rmcp::model::{CallToolRequestParams, JsonObject};
    use serde_json::{Value, json};

    use crate::{config::Config, mcp::tools, stash::StashClient};

    // ── Test helpers ──────────────────────────────────────────────────────────

    fn load_config() -> Option<Config> {
        let path = Path::new(".env");
        if !path.exists() {
            return None;
        }
        Config::from_file(path).ok()
    }

    /// Skip the test if no `.env` is found; otherwise return (config, client).
    macro_rules! require_live {
        ($cfg:ident, $cli:ident) => {
            let Some($cfg) = load_config() else {
                eprintln!("[integration] no .env found — skipping");
                return;
            };
            let $cli = StashClient::new(&$cfg);
        };
    }

    fn tool_req(name: &str, args: Value) -> CallToolRequestParams {
        let obj: JsonObject = serde_json::from_value(args).unwrap();
        CallToolRequestParams::new(name.to_owned()).with_arguments(obj)
    }

    fn tool_req_noargs(name: &str) -> CallToolRequestParams {
        CallToolRequestParams::new(name.to_owned())
    }

    /// Extract the text body from the first Content item in a tool result.
    fn first_text(result: &rmcp::model::CallToolResult) -> &str {
        result
            .content
            .first()
            .and_then(|c| c.as_text())
            .map(|t| t.text.as_str())
            .unwrap_or("")
    }

    // ── Health ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn integration_health_check_returns_version() {
        require_live!(cfg, cli);
        let ver = cli.health_check().await.expect("health_check failed");
        assert!(!ver.version.is_empty(), "version string must be non-empty");
    }

    // ── Performers ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn integration_find_performers_returns_list() {
        require_live!(_cfg, cli);
        let performers = cli
            .find_performers(Value::Null)
            .await
            .expect("find_performers failed");
        assert!(
            !performers.is_empty(),
            "expected at least one performer in the library"
        );
        for p in &performers {
            assert!(!p.id.is_empty(), "performer id must not be empty");
            assert!(!p.name.is_empty(), "performer name must not be empty");
        }
    }

    #[tokio::test]
    async fn integration_find_performers_favorites_only() {
        require_live!(_cfg, cli);
        use crate::stash::filters::PerformerFilterBuilder;
        let filter = PerformerFilterBuilder::new().favorites_only(true).build();
        let favorites = cli
            .find_performers(filter)
            .await
            .expect("find_performers(favorites) failed");
        for p in &favorites {
            assert!(
                p.favorite,
                "performer '{}' returned by favorites filter but favorite=false",
                p.name
            );
        }
    }

    #[tokio::test]
    async fn integration_find_performer_by_name_roundtrip() {
        require_live!(_cfg, cli);
        let all = cli
            .find_performers(Value::Null)
            .await
            .expect("list performers failed");
        let Some(first) = all.into_iter().next() else {
            eprintln!("[integration] no performers — skipping roundtrip");
            return;
        };
        let found = cli
            .find_performer_by_name(&first.name)
            .await
            .expect("find_performer_by_name failed")
            .expect("performer not found by name");
        assert_eq!(
            found.id, first.id,
            "find_performer_by_name returned a different id"
        );
    }

    // ── Scenes ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn integration_find_scenes_returns_list() {
        require_live!(_cfg, cli);
        let scenes = cli
            .find_scenes(Value::Null)
            .await
            .expect("find_scenes failed");
        assert!(
            !scenes.is_empty(),
            "expected at least one scene in the library"
        );
        for s in &scenes {
            assert!(!s.id.is_empty(), "scene id must not be empty");
        }
    }

    #[tokio::test]
    async fn integration_find_scenes_organized_filter() {
        require_live!(_cfg, cli);
        use crate::stash::filters::SceneFilterBuilder;
        let filter = SceneFilterBuilder::new().organized_only(true).build();
        let scenes = cli
            .find_scenes(filter)
            .await
            .expect("find_scenes(organized) failed");
        for s in &scenes {
            assert!(
                s.organized,
                "scene '{}' returned by organized filter but organized=false",
                s.id
            );
        }
    }

    #[tokio::test]
    async fn integration_find_scenes_for_performer() {
        require_live!(_cfg, cli);
        let all_performers = cli
            .find_performers(Value::Null)
            .await
            .expect("list performers failed");
        let Some(performer) = all_performers.into_iter().next() else {
            eprintln!("[integration] no performers — skipping scenes_for_performer");
            return;
        };
        // organized_only=false so we get all scenes regardless of state
        let scenes = cli
            .find_scenes_for_performer(&performer.name, false)
            .await
            .expect("find_scenes_for_performer failed");
        // Every returned scene must reference the performer
        for s in &scenes {
            let has_performer = s
                .performers
                .iter()
                .flatten()
                .any(|p| p.name == performer.name);
            assert!(
                has_performer,
                "scene '{}' does not reference performer '{}'",
                s.id, performer.name
            );
        }
    }

    // ── Studios ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn integration_find_studios_returns_list() {
        require_live!(_cfg, cli);
        let studios = cli
            .find_studios(Value::Null)
            .await
            .expect("find_studios failed");
        for s in &studios {
            assert!(!s.id.is_empty(), "studio id must not be empty");
            assert!(!s.name.is_empty(), "studio name must not be empty");
        }
    }

    #[tokio::test]
    async fn integration_find_studio_by_name_roundtrip() {
        require_live!(_cfg, cli);
        let all = cli
            .find_studios(Value::Null)
            .await
            .expect("list studios failed");
        let Some(first) = all.into_iter().next() else {
            eprintln!("[integration] no studios — skipping roundtrip");
            return;
        };
        let found = cli
            .find_studio_by_name(&first.name)
            .await
            .expect("find_studio_by_name failed")
            .expect("studio not found by name");
        assert_eq!(
            found.id, first.id,
            "find_studio_by_name returned a different id"
        );
    }

    // ── Tags ──────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn integration_find_tags_returns_list() {
        require_live!(_cfg, cli);
        let tags = cli.find_tags(Value::Null).await.expect("find_tags failed");
        assert!(!tags.is_empty(), "expected at least one tag in the library");
        for t in &tags {
            assert!(!t.id.is_empty(), "tag id must not be empty");
            assert!(!t.name.is_empty(), "tag name must not be empty");
        }
    }

    #[tokio::test]
    async fn integration_find_tag_by_name_roundtrip() {
        require_live!(_cfg, cli);
        let all = cli.find_tags(Value::Null).await.expect("list tags failed");
        let Some(first) = all.into_iter().next() else {
            eprintln!("[integration] no tags — skipping roundtrip");
            return;
        };
        let found = cli
            .find_tag_by_name(&first.name)
            .await
            .expect("find_tag_by_name failed")
            .expect("tag not found by name");
        assert_eq!(
            found.id, first.id,
            "find_tag_by_name returned a different id"
        );
    }

    #[tokio::test]
    async fn integration_resolve_tag_ids_returns_correct_ids() {
        require_live!(_cfg, cli);
        let all = cli.find_tags(Value::Null).await.expect("list tags failed");
        let sample: Vec<_> = all.iter().take(2).collect();
        if sample.is_empty() {
            eprintln!("[integration] no tags — skipping resolve_tag_ids");
            return;
        }
        let names: Vec<&str> = sample.iter().map(|t| t.name.as_str()).collect();
        let expected_ids: Vec<&str> = sample.iter().map(|t| t.id.as_str()).collect();

        let resolved = cli
            .resolve_tag_ids(&names)
            .await
            .expect("resolve_tag_ids failed");
        assert_eq!(
            resolved.len(),
            names.len(),
            "resolved count must match input count"
        );
        for id in &resolved {
            assert!(
                expected_ids.contains(&id.as_str()),
                "resolved id '{id}' not in expected set"
            );
        }
    }

    #[tokio::test]
    async fn integration_resolve_unknown_tag_returns_error() {
        require_live!(_cfg, cli);
        let result = cli.resolve_tag_ids(&["__NO_SUCH_TAG_ZZZZ__"]).await;
        assert!(result.is_err(), "expected error for unknown tag name");
    }

    // ── Tool dispatch ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn integration_tool_health_check_shows_connected() {
        require_live!(cfg, cli);
        let req = tool_req_noargs("health_check");
        let result = tools::call_tool(&cli, &cfg, req)
            .await
            .expect("call_tool failed");
        let text = first_text(&result);
        let v: Value = serde_json::from_str(text).expect("result is not valid JSON");
        assert_eq!(
            v["connected"], true,
            "health_check must report connected=true; got: {text}"
        );
        assert!(
            v["stash_version"].is_string(),
            "health_check must include stash_version"
        );
    }

    #[tokio::test]
    async fn integration_tool_get_all_performers_returns_array() {
        require_live!(cfg, cli);
        let req = tool_req("get_all_performers", json!({ "favorites_only": false }));
        let result = tools::call_tool(&cli, &cfg, req)
            .await
            .expect("call_tool failed");
        let text = first_text(&result);
        let v: Value = serde_json::from_str(text).expect("result is not valid JSON");
        assert!(v.is_array(), "expected JSON array, got: {text}");
        let arr = v.as_array().unwrap();
        for item in arr {
            assert!(item["id"].is_string(), "each performer must have an id");
            assert!(item["name"].is_string(), "each performer must have a name");
            assert!(item["link"].is_string(), "each performer must have a link");
        }
    }

    #[tokio::test]
    async fn integration_tool_get_performer_info_found() {
        require_live!(cfg, cli);
        let all = cli
            .find_performers(Value::Null)
            .await
            .expect("list performers failed");
        let Some(p) = all.into_iter().next() else {
            eprintln!("[integration] no performers — skipping");
            return;
        };
        let req = tool_req("get_performer_info", json!({ "performer_name": p.name }));
        let result = tools::call_tool(&cli, &cfg, req)
            .await
            .expect("call_tool failed");
        let text = first_text(&result);
        let v: Value = serde_json::from_str(text).expect("result is not valid JSON");
        assert_eq!(v["id"].as_str(), Some(p.id.as_str()), "id mismatch");
        assert!(v["link"].is_string(), "link must be present");
    }

    #[tokio::test]
    async fn integration_tool_get_performer_info_not_found_returns_error_content() {
        require_live!(cfg, cli);
        let req = tool_req(
            "get_performer_info",
            json!({ "performer_name": "__NO_SUCH_PERFORMER_ZZZZ__" }),
        );
        let result = tools::call_tool(&cli, &cfg, req)
            .await
            .expect("call_tool dispatch must not fail");
        assert!(result.is_error.unwrap_or(false), "expected is_error=true");
    }

    #[tokio::test]
    async fn integration_tool_get_all_scenes_returns_array() {
        require_live!(cfg, cli);
        let req = tool_req("get_all_scenes", json!({ "organized_only": false }));
        let result = tools::call_tool(&cli, &cfg, req)
            .await
            .expect("call_tool failed");
        let text = first_text(&result);
        let v: Value = serde_json::from_str(text).expect("result is not valid JSON");
        assert!(v.is_array(), "expected JSON array, got: {text}");
        let arr = v.as_array().unwrap();
        for item in arr {
            assert!(item["id"].is_string(), "each scene must have an id");
        }
    }

    #[tokio::test]
    async fn integration_tool_get_all_scenes_from_performer() {
        require_live!(cfg, cli);
        let all = cli
            .find_performers(Value::Null)
            .await
            .expect("list performers failed");
        let Some(p) = all.into_iter().next() else {
            eprintln!("[integration] no performers — skipping");
            return;
        };
        let req = tool_req(
            "get_all_scenes_from_performer",
            json!({ "performer_name": p.name, "organized_only": false }),
        );
        let result = tools::call_tool(&cli, &cfg, req)
            .await
            .expect("call_tool failed");
        assert!(
            !result.is_error.unwrap_or(false),
            "unexpected error: {}",
            first_text(&result)
        );
        let text = first_text(&result);
        let v: Value = serde_json::from_str(text).expect("result is not valid JSON");
        assert!(v.is_array(), "expected JSON array, got: {text}");
    }

    #[tokio::test]
    async fn integration_tool_advanced_performer_analysis() {
        require_live!(cfg, cli);
        let all = cli
            .find_performers(Value::Null)
            .await
            .expect("list performers failed");
        let Some(p) = all.into_iter().next() else {
            eprintln!("[integration] no performers — skipping");
            return;
        };
        let req = tool_req(
            "advanced_performer_analysis",
            json!({
                "performer_name": p.name,
                "include_similar": false,
                "deep_scene_analysis": true
            }),
        );
        let result = tools::call_tool(&cli, &cfg, req)
            .await
            .expect("call_tool failed");
        assert!(
            !result.is_error.unwrap_or(false),
            "unexpected error: {}",
            first_text(&result)
        );
        let text = first_text(&result);
        let v: Value = serde_json::from_str(text).expect("result is not valid JSON");
        assert!(
            v["performer_info"].is_object(),
            "must have performer_info field"
        );
        assert!(
            v["scene_statistics"].is_object(),
            "must have scene_statistics field"
        );
        assert!(
            v["detailed_scene_analysis"].is_object(),
            "deep_scene_analysis=true must produce detailed_scene_analysis"
        );
        assert!(
            v["detailed_scene_analysis"]["scenes_by_rating"].is_object(),
            "detailed_scene_analysis must have scenes_by_rating"
        );
    }

    #[tokio::test]
    async fn integration_tool_batch_performer_insights() {
        require_live!(cfg, cli);
        let all = cli
            .find_performers(Value::Null)
            .await
            .expect("list performers failed");
        let names: Vec<&str> = all.iter().take(3).map(|p| p.name.as_str()).collect();
        if names.is_empty() {
            eprintln!("[integration] no performers — skipping");
            return;
        }
        let req = tool_req(
            "batch_performer_insights",
            json!({ "performer_names": names }),
        );
        let result = tools::call_tool(&cli, &cfg, req)
            .await
            .expect("call_tool failed");
        assert!(
            !result.is_error.unwrap_or(false),
            "unexpected error: {}",
            first_text(&result)
        );
        let text = first_text(&result);
        let v: Value = serde_json::from_str(text).expect("result is not valid JSON");
        assert!(v["summary"].is_object(), "must have summary field");
        assert_eq!(
            v["summary"]["total_requested"].as_u64(),
            Some(names.len() as u64),
            "total_requested must match input length"
        );
        assert!(v["performers"].is_array(), "must have performers array");
    }

    #[tokio::test]
    async fn integration_tool_unknown_name_returns_mcp_error() {
        require_live!(cfg, cli);
        let req = tool_req_noargs("__no_such_tool__");
        let result = tools::call_tool(&cli, &cfg, req).await;
        assert!(result.is_err(), "unknown tool must return Err(McpError)");
    }
}
