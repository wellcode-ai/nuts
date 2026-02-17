use miette::Diagnostic;
use thiserror::Error;

/// Unified error type for NUTS.
///
/// Each variant carries enough context for `miette` to render rich diagnostics
/// with help text and suggestions.
#[derive(Debug, Error, Diagnostic)]
pub enum NutsError {
    #[error("HTTP request failed: {message}")]
    #[diagnostic(
        code(nuts::http),
        help(
            "Check the URL and your network connection. Try: nuts call GET https://httpbin.org/get"
        )
    )]
    Http {
        message: String,
        #[source]
        source: Option<reqwest::Error>,
    },

    #[error("AI service error: {message}")]
    #[diagnostic(
        code(nuts::ai),
        help("Ensure your API key is set: nuts config set api-key <KEY>")
    )]
    Ai { message: String },

    #[error("Configuration error: {message}")]
    #[diagnostic(
        code(nuts::config),
        help("Run 'nuts config show' to inspect current configuration")
    )]
    Config { message: String },

    #[error("MCP protocol error: {message}")]
    #[diagnostic(
        code(nuts::mcp),
        help("Verify the MCP server is running and the transport is correct")
    )]
    Mcp { message: String },

    #[error("Protocol error: {message}")]
    #[diagnostic(code(nuts::protocol))]
    Protocol { message: String },

    #[error("Flow error: {message}")]
    #[diagnostic(code(nuts::flow), help("Run 'nuts flow list' to see available flows"))]
    Flow { message: String },

    #[error("Authentication required: {message}")]
    #[diagnostic(
        code(nuts::auth),
        help("Provide credentials with --bearer <TOKEN> or -u user:pass")
    )]
    AuthRequired { message: String },

    #[error(transparent)]
    #[diagnostic(code(nuts::io))]
    Io(#[from] std::io::Error),

    #[error("Invalid input: {message}")]
    #[diagnostic(code(nuts::input))]
    InvalidInput { message: String },
}

/// Convenience alias used throughout the codebase.
pub type Result<T> = std::result::Result<T, NutsError>;

// ---------------------------------------------------------------------------
// Conversions from common external error types
// ---------------------------------------------------------------------------

impl From<reqwest::Error> for NutsError {
    fn from(err: reqwest::Error) -> Self {
        NutsError::Http {
            message: err.to_string(),
            source: Some(err),
        }
    }
}

impl From<serde_json::Error> for NutsError {
    fn from(err: serde_json::Error) -> Self {
        NutsError::InvalidInput {
            message: format!("JSON error: {err}"),
        }
    }
}

impl From<serde_yaml::Error> for NutsError {
    fn from(err: serde_yaml::Error) -> Self {
        NutsError::InvalidInput {
            message: format!("YAML error: {err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_error_displays_message() {
        let err = NutsError::Http {
            message: "connection refused".into(),
            source: None,
        };
        assert!(err.to_string().contains("connection refused"));
    }

    #[test]
    fn io_error_converts() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let nuts_err: NutsError = io_err.into();
        assert!(nuts_err.to_string().contains("file missing"));
    }

    #[test]
    fn result_alias_works() {
        fn example() -> Result<u32> {
            Ok(42)
        }
        assert_eq!(example().unwrap(), 42);
    }
}
