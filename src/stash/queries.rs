/// Queries all fields used for the health check.
pub const HEALTH_CHECK: &str = "{ version { version build_time } }";

/// Query to list/search performers.
/// Variables: `$filter: FindFilterType`, `$performer_filter: PerformerFilterType`
pub const FIND_PERFORMERS: &str = r#"
query FindPerformers($filter: FindFilterType, $performer_filter: PerformerFilterType) {
  findPerformers(filter: $filter, performer_filter: $performer_filter) {
    count
    performers {
      id name favorite country details ethnicity eye_color hair_color
      height_cm measurements piercings tattoos weight
      tags { id name }
    }
  }
}
"#;

/// Query to list/search scenes.
/// Variables: `$filter: FindFilterType`, `$scene_filter: SceneFilterType`
pub const FIND_SCENES: &str = r#"
query FindScenes($filter: FindFilterType, $scene_filter: SceneFilterType) {
  findScenes(filter: $filter, scene_filter: $scene_filter) {
    count
    scenes {
      id title details organized rating100
      performers { name rating100 tags { id name } }
      tags { id name }
    }
  }
}
"#;

/// Query to list/search studios.
/// Variables: `$filter: FindFilterType`, `$studio_filter: StudioFilterType`
pub const FIND_STUDIOS: &str = r#"
query FindStudios($filter: FindFilterType, $studio_filter: StudioFilterType) {
  findStudios(filter: $filter, studio_filter: $studio_filter) {
    count
    studios {
      id name url details rating100 favorite scene_count
      parent_studio { id name }
      child_studios { id name }
      aliases
      tags { id name }
    }
  }
}
"#;

/// Query to list/search tags.
/// Variables: `$filter: FindFilterType`, `$tag_filter: TagFilterType`
pub const FIND_TAGS: &str = r#"
query FindTags($filter: FindFilterType, $tag_filter: TagFilterType) {
  findTags(filter: $filter, tag_filter: $tag_filter) {
    count
    tags {
      id name description favorite scene_count scene_marker_count
      aliases
      parents { id name }
      children { id name }
    }
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_are_non_empty() {
        assert!(!HEALTH_CHECK.is_empty());
        assert!(!FIND_PERFORMERS.is_empty());
        assert!(!FIND_SCENES.is_empty());
        assert!(!FIND_STUDIOS.is_empty());
        assert!(!FIND_TAGS.is_empty());
    }

    #[test]
    fn find_performers_contains_expected_fields() {
        assert!(FIND_PERFORMERS.contains("findPerformers"));
        assert!(FIND_PERFORMERS.contains("performer_filter"));
        assert!(FIND_PERFORMERS.contains("height_cm"));
        assert!(FIND_PERFORMERS.contains("ethnicity"));
        assert!(FIND_PERFORMERS.contains("favorite"));
    }

    #[test]
    fn find_scenes_contains_expected_fields() {
        assert!(FIND_SCENES.contains("findScenes"));
        assert!(FIND_SCENES.contains("scene_filter"));
        assert!(FIND_SCENES.contains("organized"));
        assert!(FIND_SCENES.contains("rating100"));
        assert!(FIND_SCENES.contains("performers"));
    }

    #[test]
    fn find_studios_contains_expected_fields() {
        assert!(FIND_STUDIOS.contains("findStudios"));
        assert!(FIND_STUDIOS.contains("parent_studio"));
        assert!(FIND_STUDIOS.contains("child_studios"));
        assert!(FIND_STUDIOS.contains("scene_count"));
    }

    #[test]
    fn find_tags_contains_expected_fields() {
        assert!(FIND_TAGS.contains("findTags"));
        assert!(FIND_TAGS.contains("scene_marker_count"));
        assert!(FIND_TAGS.contains("parents"));
        assert!(FIND_TAGS.contains("children"));
    }
}
