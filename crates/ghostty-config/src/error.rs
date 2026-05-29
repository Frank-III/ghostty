//! Configuration errors and diagnostics (`src/config/ErrorList.zig`, CLI diags).

use std::fmt;

/// Base parse/load errors for config values and files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    ValueRequired,
    InvalidField,
    InvalidValue,
    Io,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValueRequired => f.write_str("value required"),
            Self::InvalidField => f.write_str("invalid field"),
            Self::InvalidValue => f.write_str("invalid value"),
            Self::Io => f.write_str("io error"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// File open/read failures mirroring `src/config/file_load.zig` `OpenFileError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadError {
    FileNotFound,
    FileIsEmpty,
    FileOpenFailed,
    NotAFile,
    NotAbsolute,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNotFound => f.write_str("file not found"),
            Self::FileIsEmpty => f.write_str("file is empty"),
            Self::FileOpenFailed => f.write_str("file open failed"),
            Self::NotAFile => f.write_str("not a file"),
            Self::NotAbsolute => f.write_str("path must be absolute"),
        }
    }
}

impl std::error::Error for LoadError {}

/// One diagnostic message (`ErrorList.Error` / CLI `Diagnostic`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub key: Option<String>,
    pub message: String,
    pub source: Option<SourceLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub path: String,
    pub line: usize,
}

/// Accumulated non-fatal parse issues (`Config._diagnostics`).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DiagnosticList {
    items: Vec<Diagnostic>,
}

impl DiagnosticList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn items(&self) -> &[Diagnostic] {
        &self.items
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.items.push(diagnostic);
    }

    pub fn invalid_field(&mut self, key: impl Into<String>, source: Option<SourceLocation>) {
        self.push(Diagnostic {
            key: Some(key.into()),
            message: "invalid field".into(),
            source,
        });
    }

    pub fn invalid_value(
        &mut self,
        key: impl Into<String>,
        message: impl Into<String>,
        source: Option<SourceLocation>,
    ) {
        self.push(Diagnostic {
            key: Some(key.into()),
            message: message.into(),
            source,
        });
    }
}
