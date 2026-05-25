use serde_json::{Value, json};

/// Returns a `FindFilterType` that requests all results (no pagination).
pub fn all_pages() -> Value {
    json!({ "per_page": -1 })
}

/// Returns a `StringCriterionInput`.
pub fn string_criterion(value: &str, modifier: &str) -> Value {
    json!({ "value": value, "modifier": modifier })
}

/// Returns an `IntCriterionInput` with a single value.
pub fn int_criterion(value: i64, modifier: &str) -> Value {
    json!({ "value": value, "modifier": modifier })
}

/// Returns an `IntCriterionInput` with two values (for BETWEEN / NOT_BETWEEN).
pub fn int_criterion_between(value: i64, value2: i64) -> Value {
    json!({ "value": value, "value2": value2, "modifier": "BETWEEN" })
}

/// Returns a `HierarchicalMultiCriterionInput` for tag filtering.
pub fn tags_criterion(tag_ids: &[String], modifier: &str) -> Value {
    json!({ "value": tag_ids, "modifier": modifier })
}

/// Returns a `MultiCriterionInput` for performer filtering inside scenes.
pub fn performers_criterion(performer_ids: &[&str], modifier: &str) -> Value {
    json!({ "value": performer_ids, "modifier": modifier })
}

// ── PerformerFilterBuilder ────────────────────────────────────────────────────

/// Builds a `PerformerFilterType` JSON object incrementally.
#[derive(Debug, Default, Clone)]
pub struct PerformerFilterBuilder {
    inner: serde_json::Map<String, Value>,
}

impl PerformerFilterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn favorites_only(mut self, yes: bool) -> Self {
        if yes {
            self.inner
                .insert("filter_favorites".to_owned(), json!(true));
        }
        self
    }

    pub fn country(mut self, value: &str, modifier: &str) -> Self {
        self.inner
            .insert("country".to_owned(), string_criterion(value, modifier));
        self
    }

    pub fn ethnicity(mut self, value: &str, modifier: &str) -> Self {
        self.inner
            .insert("ethnicity".to_owned(), string_criterion(value, modifier));
        self
    }

    pub fn eye_color(mut self, value: &str, modifier: &str) -> Self {
        self.inner
            .insert("eye_color".to_owned(), string_criterion(value, modifier));
        self
    }

    pub fn hair_color(mut self, value: &str, modifier: &str) -> Self {
        self.inner
            .insert("hair_color".to_owned(), string_criterion(value, modifier));
        self
    }

    pub fn height_cm(mut self, value: i64, modifier: &str, value2: Option<i64>) -> Self {
        let criterion = match value2 {
            Some(v2) if modifier == "BETWEEN" || modifier == "NOT_BETWEEN" => {
                int_criterion_between(value, v2)
            }
            _ => int_criterion(value, modifier),
        };
        self.inner.insert("height_cm".to_owned(), criterion);
        self
    }

    pub fn measurements(mut self, value: &str, modifier: &str) -> Self {
        self.inner
            .insert("measurements".to_owned(), string_criterion(value, modifier));
        self
    }

    pub fn piercings(mut self, value: &str) -> Self {
        self.inner
            .insert("piercings".to_owned(), string_criterion(value, "INCLUDES"));
        self
    }

    pub fn tattoos(mut self, value: &str) -> Self {
        self.inner
            .insert("tattoos".to_owned(), string_criterion(value, "INCLUDES"));
        self
    }

    pub fn weight(mut self, value: i64, modifier: &str, value2: Option<i64>) -> Self {
        let criterion = match value2 {
            Some(v2) if modifier == "BETWEEN" || modifier == "NOT_BETWEEN" => {
                int_criterion_between(value, v2)
            }
            _ => int_criterion(value, modifier),
        };
        self.inner.insert("weight".to_owned(), criterion);
        self
    }

    /// Name equality match — used to resolve a performer by exact name.
    pub fn name_equals(mut self, name: &str) -> Self {
        self.inner
            .insert("name".to_owned(), string_criterion(name, "EQUALS"));
        self
    }

    pub fn build(self) -> Value {
        Value::Object(self.inner)
    }
}

// ── SceneFilterBuilder ────────────────────────────────────────────────────────

/// Builds a `SceneFilterType` JSON object incrementally.
#[derive(Debug, Default, Clone)]
pub struct SceneFilterBuilder {
    inner: serde_json::Map<String, Value>,
}

impl SceneFilterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn organized_only(mut self, yes: bool) -> Self {
        if yes {
            self.inner.insert("organized".to_owned(), json!(true));
        }
        self
    }

    /// Only scenes with at least one tag.
    pub fn has_tags(mut self) -> Self {
        self.inner
            .insert("tag_count".to_owned(), int_criterion(0, "GREATER_THAN"));
        self
    }

    pub fn min_rating(mut self, min: i64) -> Self {
        // GREATER_THAN is exclusive, so subtract 1 to make it inclusive.
        self.inner.insert(
            "rating100".to_owned(),
            int_criterion(min - 1, "GREATER_THAN"),
        );
        self
    }

    pub fn max_rating(mut self, max: i64) -> Self {
        // LESS_THAN is exclusive, so add 1 to make it inclusive.
        self.inner
            .insert("rating100".to_owned(), int_criterion(max + 1, "LESS_THAN"));
        self
    }

    pub fn rating_range(mut self, min: i64, max: i64) -> Self {
        self.inner
            .insert("rating100".to_owned(), int_criterion_between(min, max));
        self
    }

    pub fn include_tags(mut self, tag_ids: &[String]) -> Self {
        self.inner
            .insert("tags".to_owned(), tags_criterion(tag_ids, "INCLUDES"));
        self
    }

    pub fn exclude_tags(mut self, tag_ids: &[String]) -> Self {
        self.inner
            .insert("tags".to_owned(), tags_criterion(tag_ids, "EXCLUDES"));
        self
    }

    /// Filter scenes that include a specific performer (by database ID).
    pub fn performer_id(mut self, id: &str) -> Self {
        self.inner.insert(
            "performers".to_owned(),
            performers_criterion(&[id], "INCLUDES"),
        );
        self
    }

    pub fn build(self) -> Value {
        Value::Object(self.inner)
    }
}

// ── StudioFilterBuilder ───────────────────────────────────────────────────────

/// Builds a `StudioFilterType` JSON object incrementally.
#[derive(Debug, Default, Clone)]
pub struct StudioFilterBuilder {
    inner: serde_json::Map<String, Value>,
}

impl StudioFilterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn favorites_only(mut self, yes: bool) -> Self {
        if yes {
            self.inner.insert("favorite".to_owned(), json!(true));
        }
        self
    }

    pub fn build(self) -> Value {
        Value::Object(self.inner)
    }
}

// ── TagFilterBuilder ──────────────────────────────────────────────────────────

/// Builds a `TagFilterType` JSON object incrementally.
#[derive(Debug, Default, Clone)]
pub struct TagFilterBuilder {
    inner: serde_json::Map<String, Value>,
}

impl TagFilterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn favorites_only(mut self, yes: bool) -> Self {
        if yes {
            self.inner.insert("favorite".to_owned(), json!(true));
        }
        self
    }

    pub fn name_equals(mut self, name: &str) -> Self {
        self.inner
            .insert("name".to_owned(), string_criterion(name, "EQUALS"));
        self
    }

    pub fn build(self) -> Value {
        Value::Object(self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────

    #[test]
    fn all_pages_has_per_page_minus_one() {
        let v = all_pages();
        assert_eq!(v["per_page"], json!(-1));
    }

    #[test]
    fn string_criterion_structure() {
        let v = string_criterion("US", "EQUALS");
        assert_eq!(v["value"], "US");
        assert_eq!(v["modifier"], "EQUALS");
    }

    #[test]
    fn int_criterion_structure() {
        let v = int_criterion(170, "GREATER_THAN");
        assert_eq!(v["value"], 170);
        assert_eq!(v["modifier"], "GREATER_THAN");
    }

    #[test]
    fn int_criterion_between_structure() {
        let v = int_criterion_between(160, 180);
        assert_eq!(v["value"], 160);
        assert_eq!(v["value2"], 180);
        assert_eq!(v["modifier"], "BETWEEN");
    }

    #[test]
    fn tags_criterion_structure() {
        let ids = vec!["1".to_owned(), "2".to_owned()];
        let v = tags_criterion(&ids, "INCLUDES");
        assert_eq!(v["modifier"], "INCLUDES");
        assert!(v["value"].is_array());
    }

    // ── PerformerFilterBuilder ────────────────────────────────────────────

    #[test]
    fn performer_filter_empty_by_default() {
        let v = PerformerFilterBuilder::new().build();
        assert_eq!(v, json!({}));
    }

    #[test]
    fn performer_filter_favorites_only() {
        let v = PerformerFilterBuilder::new().favorites_only(true).build();
        assert_eq!(v["filter_favorites"], json!(true));
    }

    #[test]
    fn performer_filter_favorites_only_false_not_set() {
        let v = PerformerFilterBuilder::new().favorites_only(false).build();
        assert!(v.get("filter_favorites").is_none());
    }

    #[test]
    fn performer_filter_country() {
        let v = PerformerFilterBuilder::new()
            .country("US", "EQUALS")
            .build();
        assert_eq!(v["country"]["value"], "US");
        assert_eq!(v["country"]["modifier"], "EQUALS");
    }

    #[test]
    fn performer_filter_piercings_uses_includes() {
        let v = PerformerFilterBuilder::new().piercings("nose").build();
        assert_eq!(v["piercings"]["modifier"], "INCLUDES");
    }

    #[test]
    fn performer_filter_height_between() {
        let v = PerformerFilterBuilder::new()
            .height_cm(160, "BETWEEN", Some(175))
            .build();
        assert_eq!(v["height_cm"]["modifier"], "BETWEEN");
        assert_eq!(v["height_cm"]["value2"], 175);
    }

    #[test]
    fn performer_filter_name_equals() {
        let v = PerformerFilterBuilder::new().name_equals("Alice").build();
        assert_eq!(v["name"]["value"], "Alice");
        assert_eq!(v["name"]["modifier"], "EQUALS");
    }

    // ── SceneFilterBuilder ────────────────────────────────────────────────

    #[test]
    fn scene_filter_organized_only() {
        let v = SceneFilterBuilder::new().organized_only(true).build();
        assert_eq!(v["organized"], json!(true));
    }

    #[test]
    fn scene_filter_organized_only_false_not_set() {
        let v = SceneFilterBuilder::new().organized_only(false).build();
        assert!(v.get("organized").is_none());
    }

    #[test]
    fn scene_filter_min_rating_is_inclusive() {
        let v = SceneFilterBuilder::new().min_rating(80).build();
        // Greater than 79 → includes 80+
        assert_eq!(v["rating100"]["value"], 79);
        assert_eq!(v["rating100"]["modifier"], "GREATER_THAN");
    }

    #[test]
    fn scene_filter_max_rating_is_inclusive() {
        let v = SceneFilterBuilder::new().max_rating(90).build();
        // Less than 91 → includes up to 90
        assert_eq!(v["rating100"]["value"], 91);
        assert_eq!(v["rating100"]["modifier"], "LESS_THAN");
    }

    #[test]
    fn scene_filter_rating_range() {
        let v = SceneFilterBuilder::new().rating_range(70, 90).build();
        assert_eq!(v["rating100"]["value"], 70);
        assert_eq!(v["rating100"]["value2"], 90);
        assert_eq!(v["rating100"]["modifier"], "BETWEEN");
    }

    #[test]
    fn scene_filter_performer_id() {
        let v = SceneFilterBuilder::new().performer_id("42").build();
        assert_eq!(v["performers"]["modifier"], "INCLUDES");
        assert_eq!(v["performers"]["value"][0], "42");
    }

    #[test]
    fn scene_filter_has_tags() {
        let v = SceneFilterBuilder::new().has_tags().build();
        assert_eq!(v["tag_count"]["modifier"], "GREATER_THAN");
        assert_eq!(v["tag_count"]["value"], 0);
    }

    // ── StudioFilterBuilder ───────────────────────────────────────────────

    #[test]
    fn studio_filter_favorites() {
        let v = StudioFilterBuilder::new().favorites_only(true).build();
        assert_eq!(v["favorite"], json!(true));
    }

    #[test]
    fn studio_filter_empty() {
        let v = StudioFilterBuilder::new().build();
        assert_eq!(v, json!({}));
    }

    // ── TagFilterBuilder ──────────────────────────────────────────────────

    #[test]
    fn tag_filter_favorites() {
        let v = TagFilterBuilder::new().favorites_only(true).build();
        assert_eq!(v["favorite"], json!(true));
    }

    #[test]
    fn tag_filter_name_equals() {
        let v = TagFilterBuilder::new().name_equals("comedy").build();
        assert_eq!(v["name"]["value"], "comedy");
        assert_eq!(v["name"]["modifier"], "EQUALS");
    }
}
