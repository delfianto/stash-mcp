use thiserror::Error;

#[derive(Debug, Error)]
pub enum StashError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("GraphQL errors: {}", .0.join(", "))]
    GraphQl(Vec<String>),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Missing field '{0}' in response")]
    MissingField(&'static str),
}

impl StashError {
    pub fn not_found(what: impl Into<String>) -> Self {
        Self::NotFound(what.into())
    }

    pub fn graphql(msgs: Vec<String>) -> Self {
        Self::GraphQl(msgs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphql_error_display() {
        let err = StashError::graphql(vec!["first".into(), "second".into()]);
        assert_eq!(err.to_string(), "GraphQL errors: first, second");
    }

    #[test]
    fn test_not_found_display() {
        let err = StashError::not_found("performer 'Alice'");
        assert_eq!(err.to_string(), "Not found: performer 'Alice'");
    }

    #[test]
    fn test_missing_field_display() {
        let err = StashError::MissingField("findPerformers");
        assert_eq!(
            err.to_string(),
            "Missing field 'findPerformers' in response"
        );
    }
}
