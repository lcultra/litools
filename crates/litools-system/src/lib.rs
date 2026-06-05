pub mod adapter;
pub mod app_indexer;
pub mod clipboard;
pub mod file_indexer;
pub mod hotkey;
pub mod launcher;
pub mod pinyin;
pub mod platform;

pub use adapter::{DiscoveredApp, SystemAdapter};

#[cfg(target_os = "linux")]
pub use platform::linux::LinuxSystemAdapter as NativeSystemAdapter;
#[cfg(target_os = "macos")]
pub use platform::macos::MacosSystemAdapter as NativeSystemAdapter;
#[cfg(target_os = "windows")]
pub use platform::windows::WindowsSystemAdapter as NativeSystemAdapter;
