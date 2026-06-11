#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

use std::path::Path;

#[cfg(target_os = "macos")]
pub use macos::application_dirs;

/// 判断路径是否与应用 bundle 相关（跨平台）。
///
/// macOS 检查含 `.app` 组件的路径；Linux/Windows 后续补齐。
pub fn is_app_ext(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str().to_string_lossy().ends_with(".app"))
}
