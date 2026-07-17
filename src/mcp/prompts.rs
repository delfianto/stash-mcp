use rmcp::ErrorData as McpError;
use rmcp::model::{
    GetPromptRequestParams, GetPromptResult, ListPromptsResult, Prompt, PromptArgument,
    PromptMessage, Role,
};

pub fn list_prompts() -> ListPromptsResult {
    let mut result = ListPromptsResult::default();
    result.prompts = vec![
        Prompt::new(
            "analyze-performer",
            Some("Generates a complete analysis of a performer: statistics, popular scenes, frequent tags, and similar recommendations"),
            Some(vec![
                PromptArgument::new("performer_name")
                    .with_description("Exact name of the performer to analyze")
                    .with_required(true),
            ]),
        ),
        Prompt::new(
            "library-insights",
            Some("Generates insights about the library: trends, metadata gaps, organization recommendations"),
            None,
        ),
        Prompt::new(
            "recommend-scenes",
            Some("Based on user preferences, recommends specific scenes with explanation of why each was chosen"),
            Some(vec![
                PromptArgument::new("preferences")
                    .with_description("User preferences: tags, performer characteristics, genres, etc.")
                    .with_required(true),
            ]),
        ),
        Prompt::new(
            "discover-performers",
            Some("Discover performers based on specific user criteria"),
            Some(vec![
                PromptArgument::new("criteria")
                    .with_description("Search criteria: physical characteristics, demographics, content preferences")
                    .with_required(true),
            ]),
        ),
    ];
    result
}

pub fn get_prompt(req: GetPromptRequestParams) -> Result<GetPromptResult, McpError> {
    let args = req.arguments.unwrap_or_default();
    let get_arg = |key: &str| -> String {
        args.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned()
    };

    let text = match req.name.as_str() {
        "analyze-performer" => {
            let name = get_arg("performer_name");
            analyze_performer_text(&name)
        }
        "library-insights" => library_insights_text(),
        "recommend-scenes" => {
            let prefs = get_arg("preferences");
            recommend_scenes_text(&prefs)
        }
        "discover-performers" => {
            let criteria = get_arg("criteria");
            discover_performers_text(&criteria)
        }
        other => {
            return Err(McpError::invalid_params(
                format!("unknown prompt '{other}'"),
                None,
            ));
        }
    };

    let mut result = GetPromptResult::default();
    result.messages = vec![PromptMessage::new_text(Role::User, text)];
    Ok(result)
}

fn analyze_performer_text(name: &str) -> String {
    format!(
        r#"Completely analyze the performer '{name}' using the available Stash MCP server tools.

REQUIRED ANALYSIS:

1. **Basic Performer Information:**
   - Use `get_performer_info('{name}')` to get demographic data
   - Include: country, ethnicity, physical characteristics, measurements, tattoos, piercings

2. **Scene Analysis:**
   - Use `get_all_scenes_from_performer('{name}')` to get all their scenes
   - Calculate statistics: total scenes, average rating
   - Identify top-rated scenes (rating > 80)

3. **Tag Analysis:**
   - Extract all unique tags from their scenes
   - Identify most frequent tags (top 10)
   - Categorize tags by type (genre, position, characteristics)

4. **Similar Performers:**
   - Use `get_all_performers()` with filters based on the performer's characteristics
   - Find performers with similar country, ethnicity, or physical measurements
   - Suggest up to 5 similar performers

5. **Recommendations:**
   - Suggest standout scenes to watch first
   - Identify gaps in the collection (popular tags that are missing)
   - Recommend related searches

OUTPUT FORMAT:
- Use markdown for formatting
- Include clear numerical statistics
- Provide actionable insights
- Highlight interesting or unusual findings"#
    )
}

fn library_insights_text() -> String {
    r#"Analyze the complete Stash library and provide strategic insights using the available MCP tools.

REQUIRED ANALYSIS:

1. **Library Overview:**
   - Use `health_check()` to get connectivity and cache statistics
   - Use `get_all_performers(favorites_only=false)` to get performer statistics
   - Calculate: total performers, geographic distribution, ethnic diversity

2. **Favorites Analysis:**
   - Compare `get_all_performers(favorites_only=true)` vs all performers
   - Calculate percentage of favorites
   - Identify patterns in favorite performers (common characteristics)

3. **Content Analysis:**
   - For a sample of favorite performers, analyze their scenes using `get_all_scenes_from_performer()`
   - Identify most popular genres/tags
   - Calculate rating statistics

4. **Gap Detection:**
   - Identify underrepresented countries/ethnicities
   - Find physical characteristic ranges with few performers
   - Suggest areas to expand the collection

5. **Organization Recommendations:**
   - Suggest consistent tagging strategies
   - Identify performers that might need more attention
   - Propose useful filters for content discovery

OUTPUT FORMAT:
- Use markdown with clear sections
- Include specific statistics and percentages
- Provide actionable recommendations"#
        .to_owned()
}

fn recommend_scenes_text(preferences: &str) -> String {
    format!(
        r#"Generate personalized scene recommendations based on the following user preferences: "{preferences}"

RECOMMENDATION PROCESS:

1. **Preference Analysis:**
   - Extract key keywords from: "{preferences}"
   - Identify preferred physical characteristics
   - Detect tags/genres of interest
   - Determine if there are geographic/ethnic preferences

2. **Search for Relevant Performers:**
   - Use `get_all_performers()` with appropriate filters
   - If physical characteristics are mentioned, use them as filters
   - Prioritize favorite performers if no specific criteria

3. **Scene Analysis by Performer:**
   - For each relevant performer, use `get_all_scenes_from_performer()`
   - Filter scenes with high ratings (rating > 75)
   - Identify scenes that match preferred tags

4. **Final Selection:**
   - Select top 10 recommended scenes
   - Ensure diversity in the selection

OUTPUT FORMAT:
For each recommended scene include:
- **Scene Title**
- **Performers**: Names and brief description
- **Rating**: Numerical rating
- **Why recommended**: Specific explanation of match with preferences
- **Relevant tags**"#
    )
}

fn discover_performers_text(criteria: &str) -> String {
    format!(
        r#"Discover performers that match the following criteria: "{criteria}"

DISCOVERY PROCESS:

1. **Criteria Interpretation:**
   - Analyze criteria: "{criteria}"
   - Identify applicable filters:
     * Physical characteristics (height, weight, measurements)
     * Demographics (country, ethnicity)
     * Distinctive features (tattoos, piercings)

2. **Stratified Search:**
   - **Level 1**: Exact search with all criteria using `get_all_performers()` with full filters
   - **Level 2**: Relaxed search with main criteria only
   - **Level 3**: Exploratory search with partial criteria

3. **Analysis of Each Found Performer:**
   - Use `get_performer_info()` for detailed data
   - Use `get_all_scenes_from_performer()` to evaluate content quality

4. **Result Categorization:**
   - **Perfect Matches**: Meet all criteria
   - **Strong Matches**: Meet main criteria
   - **Interesting Discoveries**: Partial criteria but high potential

OUTPUT FORMAT:
For each performer include:
- **Performer Name**
- **Demographics**: Country, ethnicity
- **Physical Characteristics**: Height, weight, measurements, distinctive features
- **Content Statistics**: Scene count, average rating
- **Criteria Match**: How they meet the criteria
- **Recommendation Level**: High / Medium / Exploratory"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_prompts_returns_four_prompts() {
        let result = list_prompts();
        assert_eq!(result.prompts.len(), 4);
    }

    #[test]
    fn list_prompts_names_are_correct() {
        let result = list_prompts();
        let names: Vec<&str> = result.prompts.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"analyze-performer"));
        assert!(names.contains(&"library-insights"));
        assert!(names.contains(&"recommend-scenes"));
        assert!(names.contains(&"discover-performers"));
    }

    #[test]
    fn list_prompts_analyze_performer_has_required_arg() {
        let result = list_prompts();
        let p = result
            .prompts
            .iter()
            .find(|p| p.name == "analyze-performer")
            .unwrap();
        let args = p.arguments.as_ref().unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].name, "performer_name");
        assert_eq!(args[0].required, Some(true));
    }

    #[test]
    fn list_prompts_library_insights_has_no_args() {
        let result = list_prompts();
        let p = result
            .prompts
            .iter()
            .find(|p| p.name == "library-insights")
            .unwrap();
        assert!(p.arguments.is_none());
    }

    fn make_req(
        name: &str,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> GetPromptRequestParams {
        let mut req = GetPromptRequestParams::default();
        req.name = name.to_owned();
        req.arguments = args;
        req
    }

    #[test]
    fn get_prompt_analyze_performer_contains_name() {
        let mut args = serde_json::Map::new();
        args.insert("performer_name".to_owned(), "Alice".into());
        let req = make_req("analyze-performer", Some(args));
        let result = get_prompt(req).unwrap();
        assert_eq!(result.messages.len(), 1);
        let text = result.messages[0]
            .content
            .as_text()
            .expect("expected text")
            .text
            .clone();
        assert!(text.contains("Alice"));
        assert!(text.contains("get_performer_info"));
    }

    #[test]
    fn get_prompt_unknown_returns_error() {
        let req = make_req("nonexistent-prompt", None);
        assert!(get_prompt(req).is_err());
    }

    #[test]
    fn get_prompt_library_insights_works_without_args() {
        let req = make_req("library-insights", None);
        let result = get_prompt(req).unwrap();
        assert!(!result.messages.is_empty());
    }

    #[test]
    fn get_prompt_recommend_scenes_embeds_preferences() {
        let mut args = serde_json::Map::new();
        args.insert("preferences".to_owned(), "tall blonde performers".into());
        let req = make_req("recommend-scenes", Some(args));
        let result = get_prompt(req).unwrap();
        let text = result.messages[0]
            .content
            .as_text()
            .expect("expected text")
            .text
            .clone();
        assert!(text.contains("tall blonde performers"));
    }

    #[test]
    fn get_prompt_discover_performers_embeds_criteria() {
        let mut args = serde_json::Map::new();
        args.insert("criteria".to_owned(), "tattooed Japanese women".into());
        let req = make_req("discover-performers", Some(args));
        let result = get_prompt(req).unwrap();
        let text = result.messages[0]
            .content
            .as_text()
            .expect("expected text")
            .text
            .clone();
        assert!(text.contains("tattooed Japanese women"));
    }
}
