//! Error types for the OCSF Semantic Model Editor API.
//!
//! This module defines error types that can occur during API operations.
//! All errors implement proper HTTP status code mapping and JSON serialization.
//!
//! # Requirements
//! - 7.5: WHEN an API request fails, THE API_Server SHALL return appropriate
//!   HTTP status codes with error details

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// API error types for the editor server.
#[derive(Error, Debug)]
pub enum EditorApiError {
    /// Schema has not been loaded yet.
    #[error("Schema not loaded")]
    SchemaNotLoaded,

    /// Invalid field path in the OCSF schema.
    #[error("Invalid field path: {path} in class {class_uid}")]
    InvalidFieldPath { path: String, class_uid: u32 },

    /// Entity not found in the model.
    #[error("Entity not found: {name}")]
    EntityNotFound { name: String },

    /// Resource not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Validation failed with errors.
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// LLM API request failed.
    #[error("LLM request failed: {0}")]
    LLMError(String),

    /// YAML parsing error.
    #[error("YAML parse error at line {line}: {message}")]
    YamlParseError { line: usize, message: String },

    /// Invalid request body.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Internal server error.
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),

    /// Index error from ocsf-index crate.
    #[error("Index error: {0}")]
    IndexError(#[from] ocsf_index::error::IndexError),
}

impl EditorApiError {
    /// Get the HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::SchemaNotLoaded => StatusCode::SERVICE_UNAVAILABLE,
            Self::InvalidFieldPath { .. } => StatusCode::BAD_REQUEST,
            Self::EntityNotFound { .. } => StatusCode::NOT_FOUND,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::ValidationFailed(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::LLMError(_) => StatusCode::BAD_GATEWAY,
            Self::YamlParseError { .. } => StatusCode::BAD_REQUEST,
            Self::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::IndexError(e) => {
                // Map index errors to appropriate HTTP status codes
                match e {
                    ocsf_index::error::IndexError::InvalidClassUid(_) => StatusCode::BAD_REQUEST,
                    ocsf_index::error::IndexError::TableIdNotFound(_) => StatusCode::NOT_FOUND,
                    ocsf_index::error::IndexError::LineageNotFound(_) => StatusCode::NOT_FOUND,
                    ocsf_index::error::IndexError::TableNotFound(_) => StatusCode::NOT_FOUND,
                    ocsf_index::error::IndexError::StatisticsNotFound(_) => StatusCode::NOT_FOUND,
                    ocsf_index::error::IndexError::PartitionNotFound { .. } => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                }
            }
        }
    }

    /// Get the error code string for this error.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::SchemaNotLoaded => "SCHEMA_NOT_LOADED",
            Self::InvalidFieldPath { .. } => "INVALID_FIELD_PATH",
            Self::EntityNotFound { .. } => "ENTITY_NOT_FOUND",
            Self::NotFound(_) => "NOT_FOUND",
            Self::ValidationFailed(_) => "VALIDATION_FAILED",
            Self::LLMError(_) => "LLM_ERROR",
            Self::YamlParseError { .. } => "YAML_PARSE_ERROR",
            Self::InvalidRequest(_) => "INVALID_REQUEST",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::IndexError(_) => "INDEX_ERROR",
        }
    }
}

/// JSON error response body.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

/// Error details in the response.
#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
}

impl IntoResponse for EditorApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(ErrorResponse {
            error: ErrorDetails {
                code: self.error_code().to_string(),
                message: self.to_string(),
            },
        });
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            EditorApiError::SchemaNotLoaded.status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            EditorApiError::InvalidFieldPath {
                path: "test".to_string(),
                class_uid: 1001
            }
            .status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            EditorApiError::EntityNotFound {
                name: "test".to_string()
            }
            .status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            EditorApiError::ValidationFailed("test".to_string()).status_code(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            EditorApiError::LLMError("test".to_string()).status_code(),
            StatusCode::BAD_GATEWAY
        );
        assert_eq!(
            EditorApiError::YamlParseError {
                line: 1,
                message: "test".to_string()
            }
            .status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(
            EditorApiError::SchemaNotLoaded.error_code(),
            "SCHEMA_NOT_LOADED"
        );
        assert_eq!(
            EditorApiError::InvalidFieldPath {
                path: "test".to_string(),
                class_uid: 1001
            }
            .error_code(),
            "INVALID_FIELD_PATH"
        );
    }

    #[test]
    fn test_error_messages() {
        let err = EditorApiError::InvalidFieldPath {
            path: "actor.user.name".to_string(),
            class_uid: 1001,
        };
        assert_eq!(
            err.to_string(),
            "Invalid field path: actor.user.name in class 1001"
        );
    }
}
