/// 权限描述符 —— Capability 到 permission 字符串 + 信任级别的映射
pub struct CapabilityDescriptor {
    /// 对外的权限标识，None 表示 Intrinsic（免声明）
    pub permission: Option<&'static str>,
    /// 是否需要 trusted 插件身份
    pub trusted_only: bool,
}

/// 能力枚举 —— 描述"调用需要什么权限"
/// 与 API Namespace（runtime / storage / host）保持正交，互不耦合
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    /// 免声明权限（runtime.ready 等）
    Intrinsic,
    /// 插件级 KV 存储
    Storage,
    /// 视图 UI 控制
    UI,
    /// 动态命令注册
    Commands,
    /// 全局设置读
    SettingsRead,
    /// 全局设置写
    SettingsWrite,
    /// 诊断信息
    Diagnostics,
    /// 列出已安装插件
    PluginsList,
    /// 插件管理（安装/卸载/启停）
    PluginsManage,
    /// 开发者工具
    Devtools,
    /// 索引重建
    Index,
    /// 搜索源注册（WebView 插件）
    SearchProvider,
    /// 搜索执行器注册（预留）
    #[allow(dead_code)]
    SearchExecutor,
}

impl Capability {
    pub fn descriptor(&self) -> CapabilityDescriptor {
        match self {
            Capability::Intrinsic => CapabilityDescriptor {
                permission: None,
                trusted_only: false,
            },
            Capability::Storage => CapabilityDescriptor {
                permission: Some("litools-sdk:allow-storage"),
                trusted_only: false,
            },
            Capability::UI => CapabilityDescriptor {
                permission: Some("litools-sdk:allow-ui"),
                trusted_only: false,
            },
            Capability::Commands => CapabilityDescriptor {
                permission: Some("litools-sdk:allow-commands"),
                trusted_only: false,
            },
            Capability::SettingsRead => CapabilityDescriptor {
                permission: Some("litools-sdk:allow-settings-read"),
                trusted_only: false,
            },
            Capability::SettingsWrite => CapabilityDescriptor {
                permission: Some("litools-sdk:allow-settings-write"),
                trusted_only: true,
            },
            Capability::Diagnostics => CapabilityDescriptor {
                permission: Some("litools-sdk:allow-diagnostics"),
                trusted_only: true,
            },
            Capability::PluginsList => CapabilityDescriptor {
                permission: Some("litools-sdk:allow-plugins-list"),
                trusted_only: true,
            },
            Capability::PluginsManage => CapabilityDescriptor {
                permission: Some("litools-core:allow-plugins-manage"),
                trusted_only: true,
            },
            Capability::Devtools => CapabilityDescriptor {
                permission: Some("litools-core:allow-devtools"),
                trusted_only: true,
            },
            Capability::Index => CapabilityDescriptor {
                permission: Some("litools-core:allow-index"),
                trusted_only: true,
            },
            Capability::SearchProvider => CapabilityDescriptor {
                permission: Some("litools-sdk:allow-search-provider"),
                trusted_only: false,
            },
            Capability::SearchExecutor => CapabilityDescriptor {
                permission: Some("litools-sdk:allow-search-provider"),
                trusted_only: false,
            },
        }
    }
}
