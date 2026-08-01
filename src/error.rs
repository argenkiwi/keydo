//! Typed error hierarchy for keydo.

#[derive(Debug, thiserror::Error)]
pub enum KeydoError {
    /// An I/O failure reading a config file (includes the path for context).
    #[error("Failed to read {path}: {source}")]
    ConfigIo {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// A syntax or semantic error while parsing a config file.
    /// `file` is unused today (`ParseCtx::current_file` is never populated —
    /// callers already print the path themselves), so Display shows only the
    /// line number and message.
    #[error("line {line}: {msg}")]
    ConfigSyntax { file: String, line: usize, msg: String },

    /// A semantic validation error caught after parsing succeeds.
    #[error("{0}")]
    ConfigSemantic(String),

    /// A transport-level I/O error communicating with the daemon over the IPC socket.
    #[error("IPC transport error: {0}")]
    IpcTransport(std::io::Error),

    /// The daemon's IPC response contained a non-UTF-8 payload.
    #[error("IPC protocol error: non-UTF-8 payload")]
    IpcEncoding,

    /// The daemon processed the IPC message but returned a failure response.
    #[error("{0}")]
    IpcRemoteFailure(String),

    /// A catch-all for errors not yet migrated to a typed variant.
    #[error("{0}")]
    Other(String),
}

impl From<String> for KeydoError {
    fn from(s: String) -> Self {
        KeydoError::Other(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_syntax_display_includes_line_number() {
        let err = KeydoError::ConfigSyntax {
            file: String::new(),
            line: 42,
            msg: "unexpected token".to_string(),
        };
        assert_eq!(err.to_string(), "line 42: unexpected token");
    }
}
