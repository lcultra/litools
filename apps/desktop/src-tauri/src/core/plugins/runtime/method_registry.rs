use super::capability::Capability;

// ── MethodId ────────────────────────────────────────────────

/// 方法标识符 —— 消除字符串散落，全仓库统一引用
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MethodId {
    // runtime
    RuntimeReady,
    RuntimeGetInfo,
    RuntimeQueryPermission,
    // storage
    StorageGet,
    StorageSet,
    StorageRemove,
    StorageClear,
    // ui
    UIClose,
    UISetTitle,
    UIToast,
    // commands
    CommandsAdd,
    CommandsRemove,
    CommandsReplace,
    CommandsUpdate,
    // settings
    SettingsGet,
    SettingsUpdate,
    // diagnostics
    DiagnosticsGet,
    // host
    HostPluginsList,
    HostPluginsToggle,
    HostPluginsInstall,
    HostPluginsUninstall,
    HostDevtoolsOpen,
    HostIndexReload,
    // search (Phase 3)
    SearchRegisterProvider,
    SearchUnregisterProvider,
}

// ── TauriCommandId ──────────────────────────────────────────

/// 直连 Tauri command 的标识符 —— 替代字符串 "toggle_plugin" 等
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TauriCommandId {
    TogglePlugin,
    InstallPlugin,
    UninstallPlugin,
    OpenPluginDevtools,
    ReloadIndex,
}

impl TauriCommandId {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            TauriCommandId::TogglePlugin => "toggle_plugin",
            TauriCommandId::InstallPlugin => "install_plugin",
            TauriCommandId::UninstallPlugin => "uninstall_plugin",
            TauriCommandId::OpenPluginDevtools => "open_plugin_devtools",
            TauriCommandId::ReloadIndex => "reload_index",
        }
    }
}

// ── MethodHandler ───────────────────────────────────────────

/// 方法的物理分发路径
pub enum MethodHandler {
    /// 通过 SDK dispatch 分发（标准路径，走 dispatch.rs）
    SdkDispatch,
    /// 直连 Tauri command（host.* 中部分方法走此路径）
    #[allow(dead_code)]
    TauriCommand(TauriCommandId),
}

// ── MethodDescriptor ────────────────────────────────────────

/// API 方法的完整元数据 —— 插件运行时的唯一真相来源（SSOT）
#[allow(dead_code)] // 字段用于后续能力域分组、文档生成、API Explorer
pub struct MethodDescriptor {
    pub id: MethodId,
    /// dispatch key，如 "storage.get"
    pub name: &'static str,
    /// 能力域："runtime" / "storage" / "ui" / "commands" / "settings" / "diagnostics" / "host"
    pub namespace: &'static str,
    /// 所需权限能力
    pub capability: Capability,
    /// 物理分发路径（预留：后续 dispatch 重构使用）
    pub handler: MethodHandler,
    /// 人类可读描述，用于文档 / API Explorer
    pub description: &'static str,
}

// ── METHOD_REGISTRY ─────────────────────────────────────────

/// 全局方法注册表 —— SSOT，驱动 dispatch、权限、文档、SDK 导出
pub static METHOD_REGISTRY: &[MethodDescriptor] = &[
    // ── runtime ──
    MethodDescriptor {
        id: MethodId::RuntimeReady,
        name: "runtime.ready",
        namespace: "runtime",
        capability: Capability::Intrinsic,
        handler: MethodHandler::SdkDispatch,
        description: "通知宿主插件视图已就绪，返回 PluginRuntimeInfo",
    },
    MethodDescriptor {
        id: MethodId::RuntimeGetInfo,
        name: "runtime.getInfo",
        namespace: "runtime",
        capability: Capability::Intrinsic,
        handler: MethodHandler::SdkDispatch,
        description: "获取当前运行时信息",
    },
    MethodDescriptor {
        id: MethodId::RuntimeQueryPermission,
        name: "permissions.query",
        namespace: "runtime",
        capability: Capability::Intrinsic,
        handler: MethodHandler::SdkDispatch,
        description: "查询指定权限是否被授予",
    },
    // ── storage ──
    MethodDescriptor {
        id: MethodId::StorageGet,
        name: "storage.get",
        namespace: "storage",
        capability: Capability::Storage,
        handler: MethodHandler::SdkDispatch,
        description: "读取插件级 KV 存储，key ≤ 256B，value ≤ 256KB",
    },
    MethodDescriptor {
        id: MethodId::StorageSet,
        name: "storage.set",
        namespace: "storage",
        capability: Capability::Storage,
        handler: MethodHandler::SdkDispatch,
        description: "写入插件级 KV 存储",
    },
    MethodDescriptor {
        id: MethodId::StorageRemove,
        name: "storage.remove",
        namespace: "storage",
        capability: Capability::Storage,
        handler: MethodHandler::SdkDispatch,
        description: "删除指定 key 的存储条目",
    },
    MethodDescriptor {
        id: MethodId::StorageClear,
        name: "storage.clear",
        namespace: "storage",
        capability: Capability::Storage,
        handler: MethodHandler::SdkDispatch,
        description: "清空当前插件全部存储",
    },
    // ── ui ──
    MethodDescriptor {
        id: MethodId::UIClose,
        name: "ui.close",
        namespace: "ui",
        capability: Capability::UI,
        handler: MethodHandler::SdkDispatch,
        description: "关闭当前插件视图",
    },
    MethodDescriptor {
        id: MethodId::UISetTitle,
        name: "ui.setTitle",
        namespace: "ui",
        capability: Capability::UI,
        handler: MethodHandler::SdkDispatch,
        description: "设置视图标题，≤ 128B",
    },
    MethodDescriptor {
        id: MethodId::UIToast,
        name: "ui.toast",
        namespace: "ui",
        capability: Capability::UI,
        handler: MethodHandler::SdkDispatch,
        description: "显示 toast 通知",
    },
    // ── commands ──
    MethodDescriptor {
        id: MethodId::CommandsAdd,
        name: "commands.add",
        namespace: "commands",
        capability: Capability::Commands,
        handler: MethodHandler::SdkDispatch,
        description: "批量注册动态搜索命令",
    },
    MethodDescriptor {
        id: MethodId::CommandsRemove,
        name: "commands.remove",
        namespace: "commands",
        capability: Capability::Commands,
        handler: MethodHandler::SdkDispatch,
        description: "批量删除动态搜索命令",
    },
    MethodDescriptor {
        id: MethodId::CommandsReplace,
        name: "commands.replace",
        namespace: "commands",
        capability: Capability::Commands,
        handler: MethodHandler::SdkDispatch,
        description: "原子替换全部动态搜索命令",
    },
    MethodDescriptor {
        id: MethodId::CommandsUpdate,
        name: "commands.update",
        namespace: "commands",
        capability: Capability::Commands,
        handler: MethodHandler::SdkDispatch,
        description: "更新单条动态命令的部分字段",
    },
    // ── settings ──
    MethodDescriptor {
        id: MethodId::SettingsGet,
        name: "settings.get",
        namespace: "settings",
        capability: Capability::SettingsRead,
        handler: MethodHandler::SdkDispatch,
        description: "读取全局应用设置",
    },
    MethodDescriptor {
        id: MethodId::SettingsUpdate,
        name: "settings.update",
        namespace: "settings",
        capability: Capability::SettingsWrite,
        handler: MethodHandler::SdkDispatch,
        description: "更新全局应用设置（自动重建快捷键 + 应用主题）",
    },
    // ── diagnostics ──
    MethodDescriptor {
        id: MethodId::DiagnosticsGet,
        name: "diagnostics.get",
        namespace: "diagnostics",
        capability: Capability::Diagnostics,
        handler: MethodHandler::SdkDispatch,
        description: "获取诊断信息",
    },
    // ── host ──
    MethodDescriptor {
        id: MethodId::HostPluginsList,
        name: "plugins.list",
        namespace: "host",
        capability: Capability::PluginsList,
        handler: MethodHandler::SdkDispatch,
        description: "列出所有已安装插件及其元数据",
    },
    MethodDescriptor {
        id: MethodId::HostPluginsToggle,
        name: "plugins.toggle",
        namespace: "host",
        capability: Capability::PluginsManage,
        handler: MethodHandler::TauriCommand(TauriCommandId::TogglePlugin),
        description: "启用或禁用指定插件",
    },
    MethodDescriptor {
        id: MethodId::HostPluginsInstall,
        name: "plugins.install",
        namespace: "host",
        capability: Capability::PluginsManage,
        handler: MethodHandler::TauriCommand(TauriCommandId::InstallPlugin),
        description: "从文件路径安装插件",
    },
    MethodDescriptor {
        id: MethodId::HostPluginsUninstall,
        name: "plugins.uninstall",
        namespace: "host",
        capability: Capability::PluginsManage,
        handler: MethodHandler::TauriCommand(TauriCommandId::UninstallPlugin),
        description: "卸载指定插件",
    },
    MethodDescriptor {
        id: MethodId::HostDevtoolsOpen,
        name: "devtools.open",
        namespace: "host",
        capability: Capability::Devtools,
        handler: MethodHandler::TauriCommand(TauriCommandId::OpenPluginDevtools),
        description: "打开插件的 Chrome DevTools",
    },
    MethodDescriptor {
        id: MethodId::HostIndexReload,
        name: "index.reload",
        namespace: "host",
        capability: Capability::Index,
        handler: MethodHandler::TauriCommand(TauriCommandId::ReloadIndex),
        description: "重建应用索引",
    },
    // ── search ──
    MethodDescriptor {
        id: MethodId::SearchRegisterProvider,
        name: "search.registerProvider",
        namespace: "search",
        capability: Capability::SearchProvider,
        handler: MethodHandler::SdkDispatch,
        description: "注册自定义搜索源",
    },
    MethodDescriptor {
        id: MethodId::SearchUnregisterProvider,
        name: "search.unregisterProvider",
        namespace: "search",
        capability: Capability::SearchProvider,
        handler: MethodHandler::SdkDispatch,
        description: "注销自定义搜索源",
    },
];

// ── 检索辅助 ──

impl MethodDescriptor {
    /// 按 dispatch name 查找方法描述符
    pub fn find_by_name(name: &str) -> Option<&'static MethodDescriptor> {
        METHOD_REGISTRY.iter().find(|m| m.name == name)
    }
}
