use serde::{Deserialize, Serialize};

/// A slim tag reference returned inside other objects.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TagRef {
    pub id: String,
    pub name: String,
}

/// Full tag object.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Tag {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub favorite: Option<bool>,
    #[serde(default)]
    pub scene_count: Option<i64>,
    #[serde(default)]
    pub scene_marker_count: Option<i64>,
    #[serde(default)]
    pub aliases: Option<Vec<String>>,
    #[serde(default)]
    pub parents: Option<Vec<TagRef>>,
    #[serde(default)]
    pub children: Option<Vec<TagRef>>,
}

/// Performer object.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Performer {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub ethnicity: Option<String>,
    #[serde(default)]
    pub eye_color: Option<String>,
    #[serde(default)]
    pub hair_color: Option<String>,
    #[serde(default)]
    pub height_cm: Option<i64>,
    #[serde(default)]
    pub measurements: Option<String>,
    #[serde(default)]
    pub piercings: Option<String>,
    #[serde(default)]
    pub tattoos: Option<String>,
    #[serde(default)]
    pub weight: Option<i64>,
    #[serde(default)]
    pub tags: Option<Vec<TagRef>>,
}

/// Performer as it appears nested inside a Scene.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ScenePerformer {
    pub name: String,
    #[serde(default)]
    pub rating100: Option<i64>,
    #[serde(default)]
    pub tags: Option<Vec<TagRef>>,
}

/// Scene object.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Scene {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub organized: bool,
    #[serde(default)]
    pub rating100: Option<i64>,
    #[serde(default)]
    pub performers: Option<Vec<ScenePerformer>>,
    #[serde(default)]
    pub tags: Option<Vec<TagRef>>,
}

/// Slim studio reference returned inside other objects.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct StudioRef {
    pub id: String,
    pub name: String,
}

/// Full studio object.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Studio {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub rating100: Option<i64>,
    #[serde(default)]
    pub favorite: Option<bool>,
    #[serde(default)]
    pub scene_count: Option<i64>,
    #[serde(default)]
    pub parent_studio: Option<StudioRef>,
    #[serde(default)]
    pub child_studios: Option<Vec<StudioRef>>,
    #[serde(default)]
    pub aliases: Option<Vec<String>>,
    #[serde(default)]
    pub tags: Option<Vec<TagRef>>,
}

/// Stash version info (used for health check).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashVersion {
    pub version: String,
    #[serde(default)]
    pub build_time: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn performer_deserializes_with_all_optionals_missing() {
        let json = r#"{"id":"1","name":"Alice"}"#;
        let p: Performer = serde_json::from_str(json).unwrap();
        assert_eq!(p.id, "1");
        assert_eq!(p.name, "Alice");
        assert!(p.country.is_none());
        assert!(p.tags.is_none());
    }

    #[test]
    fn performer_deserializes_with_tags() {
        let json = r#"{
            "id": "1",
            "name": "Alice",
            "tags": [{"id": "10", "name": "tag1"}]
        }"#;
        let p: Performer = serde_json::from_str(json).unwrap();
        let tags = p.tags.unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "tag1");
    }

    #[test]
    fn scene_deserializes_minimal() {
        let json = r#"{"id":"5"}"#;
        let s: Scene = serde_json::from_str(json).unwrap();
        assert_eq!(s.id, "5");
        assert!(s.title.is_none());
        assert!(s.rating100.is_none());
    }

    #[test]
    fn scene_deserializes_full() {
        let json = r#"{
            "id": "5",
            "title": "Test Scene",
            "rating100": 85,
            "performers": [{"name": "Alice", "rating100": 90}],
            "tags": [{"id": "1", "name": "action"}]
        }"#;
        let s: Scene = serde_json::from_str(json).unwrap();
        assert_eq!(s.title.unwrap(), "Test Scene");
        assert_eq!(s.rating100.unwrap(), 85);
        assert_eq!(s.performers.unwrap()[0].name, "Alice");
    }

    #[test]
    fn tag_deserializes_full() {
        let json = r#"{
            "id": "3",
            "name": "comedy",
            "favorite": true,
            "scene_count": 42,
            "parents": [{"id": "1", "name": "parent"}],
            "children": [{"id": "4", "name": "sub"}]
        }"#;
        let t: Tag = serde_json::from_str(json).unwrap();
        assert!(t.favorite.unwrap());
        assert_eq!(t.scene_count.unwrap(), 42);
        assert_eq!(t.parents.unwrap()[0].name, "parent");
    }

    #[test]
    fn studio_deserializes_with_hierarchy() {
        let json = r#"{
            "id": "7",
            "name": "Acme",
            "scene_count": 100,
            "parent_studio": {"id": "2", "name": "BigCo"},
            "child_studios": [{"id": "8", "name": "Acme Sub"}]
        }"#;
        let s: Studio = serde_json::from_str(json).unwrap();
        assert_eq!(s.parent_studio.unwrap().name, "BigCo");
        assert_eq!(s.child_studios.unwrap()[0].name, "Acme Sub");
    }

    #[test]
    fn stash_version_deserializes() {
        let json = r#"{"version":"v0.27.0","build_time":"2024-01-01"}"#;
        let v: StashVersion = serde_json::from_str(json).unwrap();
        assert_eq!(v.version, "v0.27.0");
    }
}
