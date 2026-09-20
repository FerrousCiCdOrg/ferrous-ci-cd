//! Text front-ends that lower a pipeline definition into the IR.
//!
//! [`PipelineConfig`] is the intermediate representation the rest of the system
//! agrees on. This module turns YAML, TOML and JSON text into it, and turns it
//! back into YAML or JSON. It is the counterpart of
//! [`crate::domain::pipeline_dsl`], which reaches the same IR from typed Rust:
//!
//! ```text
//! YAML  ─┐
//! TOML  ─┼─→  PipelineConfig (IR)  ─→  engine / TUI / web UI
//! Rust  ─┘         ^ serde-serializable
//!  SDK
//! ```
//!
//! Loading always validates. Whatever a definition was written in, it has
//! passed the same [`PipelineConfig::validate`] by the time anything else sees
//! it.

use std::fmt;
use std::path::Path;

use crate::domain::value_objects::pipeline_config::PipelineConfig;
use crate::{Error, Result};

/// A textual encoding of the pipeline IR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineFormat {
    /// YAML, the default for hand-written pipeline files.
    Yaml,
    /// TOML, for projects that prefer it.
    Toml,
    /// JSON, the canonical encoding for machine-generated definitions.
    Json,
}

impl PipelineFormat {
    /// Pick a format from a file extension.
    ///
    /// Recognises `yaml`, `yml`, `toml` and `json`, case-insensitively.
    #[must_use]
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            "json" => Some(Self::Json),
            _ => None,
        }
    }

    /// Pick a format from a path's extension.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if the path has no extension, or one that
    /// does not name a supported format.
    pub fn from_path(path: &Path) -> Result<Self> {
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or_else(|| {
                Error::validation(format!(
                    "Cannot tell the pipeline format of '{}': the file has no extension",
                    path.display()
                ))
            })?;

        Self::from_extension(extension).ok_or_else(|| {
            Error::validation(format!(
                "Unsupported pipeline format '{extension}'. Supported: yaml, yml, toml, json"
            ))
        })
    }

    /// Whether this format can be written back out.
    ///
    /// TOML is read-only. The IR nests scalars after arrays of tables, which
    /// TOML cannot represent in the order a serializer emits them, so a
    /// round-trip through TOML would not be faithful.
    #[must_use]
    pub fn is_writable(self) -> bool {
        matches!(self, Self::Yaml | Self::Json)
    }

    /// The conventional file extension for this format.
    #[must_use]
    pub fn extension(self) -> &'static str {
        match self {
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Json => "json",
        }
    }
}

impl fmt::Display for PipelineFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.extension())
    }
}

/// Parse a pipeline definition and validate it.
///
/// # Errors
///
/// Returns [`Error::Serialization`] if the text is not well-formed in the given
/// format, or [`Error::Validation`] if the resulting configuration does not pass
/// [`PipelineConfig::validate`].
pub fn from_str(text: &str, format: PipelineFormat) -> Result<PipelineConfig> {
    let config = parse(text, format)?;
    config.validate()?;
    Ok(config)
}

/// Read a pipeline definition from disk and validate it.
///
/// The format is taken from the file extension.
///
/// # Errors
///
/// Returns [`Error::Io`] if the file cannot be read, [`Error::Validation`] if
/// the extension does not name a supported format or the configuration is
/// invalid, and [`Error::Serialization`] if the text is malformed.
pub fn from_path(path: impl AsRef<Path>) -> Result<PipelineConfig> {
    let path = path.as_ref();
    let format = PipelineFormat::from_path(path)?;
    let text = std::fs::read_to_string(path)?;
    from_str(&text, format)
}

/// Serialize a pipeline configuration.
///
/// # Errors
///
/// Returns [`Error::Validation`] if the format cannot be written (see
/// [`PipelineFormat::is_writable`]), or [`Error::Serialization`] if
/// serialization fails.
pub fn to_string(config: &PipelineConfig, format: PipelineFormat) -> Result<String> {
    match format {
        PipelineFormat::Yaml => {
            serde_yaml::to_string(config).map_err(|error| Error::serialization(error.to_string()))
        }
        PipelineFormat::Json => serde_json::to_string_pretty(config)
            .map_err(|error| Error::serialization(error.to_string())),
        PipelineFormat::Toml => Err(Error::validation(
            "TOML is a read-only pipeline format; emit yaml or json instead",
        )),
    }
}

/// Parse without validating.
fn parse(text: &str, format: PipelineFormat) -> Result<PipelineConfig> {
    match format {
        PipelineFormat::Yaml => {
            serde_yaml::from_str(text).map_err(|error| Error::serialization(error.to_string()))
        }
        PipelineFormat::Toml => {
            toml::from_str(text).map_err(|error| Error::serialization(error.to_string()))
        }
        PipelineFormat::Json => {
            serde_json::from_str(text).map_err(|error| Error::serialization(error.to_string()))
        }
    }
}
