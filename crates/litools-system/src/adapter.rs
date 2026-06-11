use std::{
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::launcher::LaunchTarget;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoveredApp {
    pub id: String,
    pub name: String,
    pub path: String,
    pub icon_path: Option<String>,
    pub localized_names: Vec<String>,
    pub aliases: Vec<String>,
    pub search_text: String,
}

/// RAII guard：drop 时自动停止应用目录监听。
pub struct AppWatchGuard {
    pub(crate) inner: Box<dyn std::any::Any + Send>,
}

impl AppWatchGuard {
    pub(crate) fn new(inner: Box<dyn std::any::Any + Send>) -> Self {
        Self { inner }
    }
}

pub trait SystemAdapter: Send + Sync {
    // === 已有 ===
    fn discover_apps(&self) -> Vec<DiscoveredApp>;

    fn launch(&self, target: &LaunchTarget) -> Result<(), String>;

    fn launch_app(&self, app_id: &str) -> Result<(), String> {
        self.launch(&LaunchTarget::App(app_id.to_string()))
    }

    fn open_file(&self, path: &str) -> Result<(), String> {
        self.launch(&LaunchTarget::File(path.to_string()))
    }

    // === 新增 ===

    /// 返回平台标准的应用安装目录列表。
    fn application_dirs(&self) -> Vec<PathBuf>;

    /// 提取应用 bundle / 可执行文件的图标，输出 PNG 字节。
    fn app_icon_png(&self, path: &Path) -> io::Result<Vec<u8>>;

    /// 开始监听应用安装目录变化。回调在 .app / .desktop 等应用 bundle
    /// 发生增删改时触发。返回 RAII guard，drop 即停止监听。
    fn watch_app_dirs(
        &self,
        on_change: Box<dyn Fn() + Send + 'static>,
    ) -> io::Result<AppWatchGuard>;
}
