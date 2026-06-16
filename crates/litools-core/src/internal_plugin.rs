use litools_plugin::PluginManifest;

use crate::extension_registry::ExtensionRegistry;

/// 内置插件的运行时表示。
///
/// 插件实例通过 `register_extensions()` 向 `ExtensionRegistry` 声明能力，
/// 不依赖 plugin.json 驱动构造。新增能力类型无需修改此 trait。
pub trait InternalPlugin: Send + Sync {
    /// 插件元数据（从 plugin.json 加载或硬编码）。
    fn metadata(&self) -> PluginManifest;

    /// 向注册表声明插件提供的所有扩展能力。
    ///
    /// 默认实现为空——无扩展能力的插件无需覆盖。
    fn register_extensions(&self, _registry: &mut ExtensionRegistry) {}
}
