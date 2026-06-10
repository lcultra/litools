use std::{collections::HashSet, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 控制插件运行时实例化策略。
///
/// 从 manifest JSON 的 `singleton: bool` 字段转换而来，内部链路统一使用此 enum，
/// 方便未来扩展 `SingletonPerWorkspace`、`Ephemeral` 等策略。
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimePolicy {
    /// 全局最多一个运行时；重新打开时若已分离则重新停靠
    Singleton,
    /// 每次打开创建新的停靠运行时，允许多实例共存
    MultiInstance,
}

impl Default for RuntimePolicy {
    fn default() -> Self {
        Self::Singleton
    }
}

/// 插件开发模式配置，仅在 manifest 中声明 `development` 时生效。
///
/// 存在 `main` 字段时，插件 webview 直接加载该 URL（通常为本地 dev server），
/// 跳过 `litools-plugin://` 协议和 `dist/` 目录。
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginDevelopment {
    /// dev server 入口 URL，如 "http://127.0.0.1:5173/index.html"
    pub main: String,
}

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
    /// 单例模式控制（JSON 字段，向后兼容默认 true）。
    ///
    /// 内部应通过 [`runtime_policy`] 转换为 [`RuntimePolicy`] enum。
    #[serde(default = "default_singleton")]
    pub singleton: bool,
    #[serde(default)]
    pub permissions: Vec<String>,
    /// 开发模式配置：指定 dev server 地址后，webview 直接加载该 URL。
    /// 仅开发阶段写入，打包发布时应移除此字段。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub development: Option<PluginDevelopment>,
}

fn default_singleton() -> bool {
    true
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
    /// 将 manifest 的 `singleton` bool 转换为 [`RuntimePolicy`] enum。
    pub fn runtime_policy(&self) -> RuntimePolicy {
        if self.singleton {
            RuntimePolicy::Singleton
        } else {
            RuntimePolicy::MultiInstance
        }
    }

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
            singleton: true,
            permissions: vec!["ui:window".to_string()],
            development: None,
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

    #[test]
    fn manifest_with_development() {
        let json = r#"{
            "id": "dev.litools.example",
            "name": "Example",
            "version": "0.1.0",
            "entry": "dist/index.html",
            "icon": "dist/icon.svg",
            "development": {
                "main": "http://127.0.0.1:5173/index.html"
            },
            "commands": []
        }"#;
        let manifest: PluginManifest = serde_json::from_str(json).expect("deserialize");
        let dev = manifest.development.expect("development field");
        assert_eq!(dev.main, "http://127.0.0.1:5173/index.html");
    }

    #[test]
    fn manifest_without_development() {
        let json = r#"{
            "id": "dev.litools.example",
            "name": "Example",
            "version": "0.1.0",
            "entry": "dist/index.html",
            "icon": "dist/icon.svg",
            "commands": []
        }"#;
        let manifest: PluginManifest = serde_json::from_str(json).expect("deserialize");
        assert!(manifest.development.is_none());
    }

    #[test]
    fn development_field_skipped_when_none() {
        let manifest = valid_manifest();
        let json = serde_json::to_string(&manifest).expect("serialize");
        assert!(!json.contains("development"));
    }
}
