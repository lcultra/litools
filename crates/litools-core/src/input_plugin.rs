use std::sync::Arc;

use litools_plugin::PluginManifest;
use litools_search::InputDetector;

use crate::detectors;
use crate::extension_registry::ExtensionRegistry;
use crate::internal_plugin::InternalPlugin;

/// 内置输入检测插件：注册所有内置 InputDetector。
///
/// 与 CommandsPlugin、LauncherPlugin、PluginHostPlugin 并列，
/// 统一走 InternalPlugin → ExtensionRegistry 注册路径。
pub struct InputPlugin;

impl InputPlugin {
    pub fn new() -> Self {
        Self
    }

    fn builtin_detectors() -> Vec<Arc<dyn InputDetector>> {
        vec![
            Arc::new(detectors::json::JsonDetector::new()),
            Arc::new(detectors::url::UrlDetector::new()),
            Arc::new(detectors::base64::Base64Detector::new()),
            Arc::new(detectors::file_path::FilePathDetector::new()),
            Arc::new(detectors::curl::CurlDetector::new()),
            Arc::new(detectors::jwt::JwtDetector::new()),
            Arc::new(detectors::uuid::UuidDetector::new()),
            Arc::new(detectors::color::ColorDetector::new()),
            Arc::new(detectors::markdown::MarkdownDetector::new()),
        ]
    }
}

impl InternalPlugin for InputPlugin {
    fn metadata(&self) -> PluginManifest {
        PluginManifest {
            id: "dev.litools.input".to_string(),
            name: "输入检测".to_string(),
            version: "1.0.0".to_string(),
            entry: None,
            description: Some("提供 JSON、URL、Base64 等内置输入特征检测能力".to_string()),
            author: Some("litools contributors".to_string()),
            icon: "input.svg".to_string(),
            commands: vec![], // 无用户可见命令，只注册 detector
            singleton: true,
            permissions: vec![],
            development: None,
        }
    }

    fn register_extensions(&self, registry: &mut ExtensionRegistry) {
        for detector in Self::builtin_detectors() {
            registry.add_input_detector(detector);
        }
    }
}
