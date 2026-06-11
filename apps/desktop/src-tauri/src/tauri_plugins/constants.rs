/// litools 内置插件的统一命名常量，单一来源维护。

pub const CORE_PLUGIN: &str = "litools-core";
pub const SDK_PLUGIN: &str = "litools-sdk";

/// ACL permission 前缀
pub const CORE_PREFIX: &str = "litools-core:";
pub const SDK_PREFIX: &str = "litools-sdk:";

/// IPC command 前缀（前端同步维护）
#[allow(dead_code)]
pub const CORE_CMD_PREFIX: &str = "plugin:litools-core|";
#[allow(dead_code)]
pub const SDK_CMD_PREFIX: &str = "plugin:litools-sdk|";
