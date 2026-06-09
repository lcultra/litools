//! 自定义 URI 协议常量。

/// 插件资源协议 scheme。插件 webview 通过 `litools-plugin://{plugin_id}/{asset}` 加载 HTML/JS/CSS。
pub const PLUGIN_PROTOCOL_SCHEME: &str = "litools-plugin";
