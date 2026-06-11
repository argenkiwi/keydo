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
    /// `file` and `line` are optional context; Display shows only `msg` so
    /// callers that already print the path don't double-print it.
    #[error("{msg}")]
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
