//! 搜索相关常量：provider 标识、结果 ID 前缀、默认限制、ResultId 类型。

// ── 搜索规模 ──

/// 启动器搜索结果最大条数，未来可由用户配置。
pub const DEFAULT_LAUNCHER_RESULT_LIMIT: usize = 20;

// ── Provider 标识 ──
//
// 每个 SearchProvider 通过 `id()` 返回一个唯一标识，搜索引擎用它匹配
// 用户设置中启用的 provider 列表。

/// 系统应用搜索 provider。
pub const APP_PROVIDER_ID: &str = "apps";
/// 插件命令搜索 provider。
pub const PLUGIN_PROVIDER_ID: &str = "plugins";
/// 内置命令搜索 provider。
pub const COMMAND_PROVIDER_ID: &str = "commands";

// ── 结果 ID 前缀 ──
//
// 搜索结果通过 `{prefix}{实体标识}` 来区分类型，方便执行时路由。

/// 应用结果 ID 前缀，格式：`app:{bundle_id}` 或 `app:path:/...`
pub const APP_RESULT_PREFIX: &str = "app:";
/// 插件结果 ID 前缀，格式：`plugin:{plugin_id}:{command_id}`
pub const PLUGIN_RESULT_PREFIX: &str = "plugin:";

// ── 操作标识 ──

/// 打开操作：启动应用、打开插件视图。
pub const ACTION_OPEN: &str = "open";
/// 执行操作：触发内置命令（如重载索引、切换主题）。
pub const ACTION_EXECUTE: &str = "execute";

// ── ResultId：类型安全的结果 ID 表示 ──
//
// 替代散落在各模块的 `*_from_result_id()` 函数，提供统一的 parse → match 路由。

/// 搜索结果 ID 的类型安全表示。
///
/// 每个搜索结果都有一个唯一的字符串 ID，按前缀区分来源：
/// - `app:{bundle_id}` → [`ResultId::App`]
/// - `plugin:{plugin_id}:{command_id}` → [`ResultId::Plugin`]
/// - 无前缀（如 `reload-index`） → [`ResultId::Builtin`]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResultId {
    App(String),
    Plugin {
        plugin_id: String,
        command_id: String,
    },
    Builtin(String),
}

impl ResultId {
    /// 将结果 ID 字符串解析为 [`ResultId`]。
    ///
    /// 按前缀匹配分派，无前缀的字符串默认归类为 `Builtin`。
    /// 返回 `None` 仅当 ID 为空或前缀后缺少必要部分。
    pub fn parse(result_id: &str) -> Option<Self> {
        if let Some(app_id) = result_id.strip_prefix(APP_RESULT_PREFIX) {
            return match app_id.is_empty() {
                true => None,
                false => Some(ResultId::App(app_id.to_string())),
            };
        }

        if let Some(rest) = result_id.strip_prefix(PLUGIN_RESULT_PREFIX) {
            let (plugin_id, command_id) = rest.rsplit_once(':')?;
            if plugin_id.is_empty() || command_id.is_empty() {
                return None;
            }
            return Some(ResultId::Plugin {
                plugin_id: plugin_id.to_string(),
                command_id: command_id.to_string(),
            });
        }

        // 无前缀的 ID 归类为内置命令，有效性由调用方验证。
        if result_id.is_empty() {
            return None;
        }
        Some(ResultId::Builtin(result_id.to_string()))
    }

    /// 将 [`ResultId`] 转换为 `pinned_items` / `usage_events` 表中的
    /// `(target_type, target_id)` 元组。
    pub fn to_target(&self) -> (&'static str, String) {
        match self {
            ResultId::App(id) => ("app", id.clone()),
            ResultId::Plugin {
                plugin_id,
                command_id,
            } => (
                crate::plugin::PLUGIN_TARGET_TYPE,
                format!("{plugin_id}:{command_id}"),
            ),
            ResultId::Builtin(id) => ("command", id.clone()),
        }
    }
}

#[cfg(test)]
mod result_id_tests {
    use super::*;

    #[test]
    fn parses_app_result_id() {
        assert_eq!(
            ResultId::parse("app:com.apple.Safari"),
            Some(ResultId::App("com.apple.Safari".to_string()))
        );
    }

    #[test]
    fn parses_plugin_result_id() {
        assert_eq!(
            ResultId::parse("plugin:dev.foo:bar"),
            Some(ResultId::Plugin {
                plugin_id: "dev.foo".to_string(),
                command_id: "bar".to_string(),
            })
        );
    }

    #[test]
    fn parses_builtin_result_id() {
        assert_eq!(
            ResultId::parse("reload-index"),
            Some(ResultId::Builtin("reload-index".to_string()))
        );
    }

    #[test]
    fn rejects_empty_id() {
        assert_eq!(ResultId::parse(""), None);
    }

    #[test]
    fn rejects_app_prefix_without_id() {
        assert_eq!(ResultId::parse("app:"), None);
    }

    #[test]
    fn rejects_plugin_prefix_without_command() {
        assert_eq!(ResultId::parse("plugin:dev.foo"), None);
    }

    #[test]
    fn to_target_app() {
        let id = ResultId::App("com.apple.Safari".to_string());
        assert_eq!(id.to_target(), ("app", "com.apple.Safari".to_string()));
    }

    #[test]
    fn to_target_plugin() {
        let id = ResultId::Plugin {
            plugin_id: "dev.foo".to_string(),
            command_id: "bar".to_string(),
        };
        assert_eq!(
            id.to_target(),
            (crate::plugin::PLUGIN_TARGET_TYPE, "dev.foo:bar".to_string())
        );
    }

    #[test]
    fn to_target_builtin() {
        let id = ResultId::Builtin("reload-index".to_string());
        assert_eq!(id.to_target(), ("command", "reload-index".to_string()));
    }
}
