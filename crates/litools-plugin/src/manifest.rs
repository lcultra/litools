use std::{collections::HashSet, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub entry: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    pub icon: String,
    #[serde(default)]
    pub commands: Vec<PluginCommand>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginCommand {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub mode: PluginCommandMode,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginCommandMode {
    Instant,
    View,
    SearchProvider,
}

#[derive(Debug, Error)]
pub enum PluginManifestError {
    #[error("plugin id is required")]
    MissingId,
    #[error("plugin id contains unsupported characters: {0}")]
    InvalidId(String),
    #[error("plugin name is required")]
    MissingName,
    #[error("plugin version is required")]
    MissingVersion,
    #[error("plugin entry is required")]
    MissingEntry,
    #[error("plugin icon is required")]
    MissingIcon,
    #[error("plugin entry must be a relative path inside the plugin directory: {0}")]
    InvalidEntry(String),
    #[error("plugin command id is required")]
    MissingCommandId,
    #[error("plugin command id contains unsupported characters: {0}")]
    InvalidCommandId(String),
    #[error("plugin command title is required: {0}")]
    MissingCommandTitle(String),
    #[error("duplicate plugin command id: {0}")]
    DuplicateCommandId(String),
}

impl PluginManifest {
    pub fn validate(&self) -> Result<(), PluginManifestError> {
        let id = self.id.trim();
        if id.is_empty() {
            return Err(PluginManifestError::MissingId);
        }
        if !is_safe_identifier(id) {
            return Err(PluginManifestError::InvalidId(self.id.clone()));
        }
        if self.name.trim().is_empty() {
            return Err(PluginManifestError::MissingName);
        }
        if self.version.trim().is_empty() {
            return Err(PluginManifestError::MissingVersion);
        }
        if self.icon.trim().is_empty() {
            return Err(PluginManifestError::MissingIcon);
        }
        validate_relative_entry(&self.entry)?;
        validate_relative_entry(&self.icon)?;

        let mut command_ids = HashSet::new();
        for command in &self.commands {
            let command_id = command.id.trim();
            if command_id.is_empty() {
                return Err(PluginManifestError::MissingCommandId);
            }
            if !is_safe_identifier(command_id) {
                return Err(PluginManifestError::InvalidCommandId(command.id.clone()));
            }
            if command.title.trim().is_empty() {
                return Err(PluginManifestError::MissingCommandTitle(command.id.clone()));
            }
            if !command_ids.insert(command_id.to_string()) {
                return Err(PluginManifestError::DuplicateCommandId(command.id.clone()));
            }
        }

        Ok(())
    }
}

impl PluginCommandMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PluginCommandMode::Instant => "instant",
            PluginCommandMode::View => "view",
            PluginCommandMode::SearchProvider => "searchProvider",
        }
    }
}

pub fn plugin_command_mode_from_str(value: &str) -> Option<PluginCommandMode> {
    match value {
        "instant" => Some(PluginCommandMode::Instant),
        "view" => Some(PluginCommandMode::View),
        "searchProvider" => Some(PluginCommandMode::SearchProvider),
        _ => None,
    }
}

fn is_safe_identifier(value: &str) -> bool {
    value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
}

fn validate_relative_entry(entry: &str) -> Result<(), PluginManifestError> {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return Err(PluginManifestError::MissingEntry);
    }

    let path = Path::new(trimmed);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(PluginManifestError::InvalidEntry(entry.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest() -> PluginManifest {
        PluginManifest {
            id: "dev.litools.example".to_string(),
            name: "Example".to_string(),
            version: "0.1.0".to_string(),
            entry: "dist/index.html".to_string(),
            description: None,
            author: None,
            icon: "dist/icon.svg".to_string(),
            commands: vec![PluginCommand {
                id: "hello".to_string(),
                title: "Hello".to_string(),
                subtitle: None,
                keywords: vec!["hello".to_string()],
                mode: PluginCommandMode::View,
            }],
            permissions: vec!["ui:window".to_string()],
        }
    }

    #[test]
    fn validates_valid_manifest() {
        valid_manifest().validate().expect("valid manifest");
    }

    #[test]
    fn rejects_entry_escape() {
        let mut manifest = valid_manifest();
        manifest.entry = "../index.html".to_string();

        assert!(matches!(
            manifest.validate(),
            Err(PluginManifestError::InvalidEntry(_))
        ));
    }

    #[test]
    fn rejects_duplicate_command_ids() {
        let mut manifest = valid_manifest();
        manifest.commands.push(manifest.commands[0].clone());

        assert!(matches!(
            manifest.validate(),
            Err(PluginManifestError::DuplicateCommandId(_))
        ));
    }
}
