use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;
use litools_plugin::RuntimePolicy;

use crate::plugin_runtime::model::{
    PluginRuntimeBounds, PluginRuntimeContext, PluginRuntimeLifecycle, PluginRuntimePlacement,
};
pub use litools_config::labels::RUNTIME_ID_PREFIX;

#[derive(Debug, Default)]
pub struct PluginRuntimeRegistry {
    next_runtime_index: u64,
    runtimes_by_id: BTreeMap<String, PluginRuntimeContext>,
    runtime_ids_by_webview_label: BTreeMap<String, String>,
    /// (plugin_id, command_id) → BTreeSet<runtime_id>
    /// 所有运行时无论 policy 都写入，Registry 只维护事实。
    runtime_ids_by_plugin_command: BTreeMap<(String, String), BTreeSet<String>>,
}

#[derive(Clone, Debug)]
pub struct PluginRuntimeRegistration {
    pub plugin_id: String,
    pub command_id: String,
    pub plugin_name: String,
    pub title: String,
    pub entry_url: String,
    pub host_window_label: String,
    pub detached_window_label: Option<String>,
    pub placement: PluginRuntimePlacement,
    pub bounds: Option<PluginRuntimeBounds>,
    pub permissions: Vec<String>,
    pub trusted: bool,
    pub policy: RuntimePolicy,
}

impl PluginRuntimeRegistry {
    fn now() -> String {
        Utc::now().to_rfc3339()
    }

    pub fn next_runtime_id(&mut self) -> String {
        self.next_runtime_index += 1;
        format!("{RUNTIME_ID_PREFIX}_{:06}", self.next_runtime_index)
    }

    pub fn register_runtime(
        &mut self,
        registration: PluginRuntimeRegistration,
        id: String,
        webview_label: String,
    ) -> Result<PluginRuntimeContext, String> {
        if self.runtimes_by_id.contains_key(&id) {
            return Err(format!("plugin runtime already exists: {id}"));
        }
        if self
            .runtime_ids_by_webview_label
            .contains_key(&webview_label)
        {
            return Err(format!(
                "plugin runtime webview label already exists: {webview_label}"
            ));
        }
        let plugin_command_key = (
            registration.plugin_id.clone(),
            registration.command_id.clone(),
        );
        // 仅 Singleton 策略检查唯一性；MultiInstance 允许多实例
        if registration.policy == RuntimePolicy::Singleton {
            if let Some(existing_ids) = self.runtime_ids_by_plugin_command.get(&plugin_command_key) {
                if !existing_ids.is_empty() {
                    return Err(format!(
                        "singleton plugin runtime already exists for command: {:?}",
                        existing_ids
                    ));
                }
            }
        }

        let now = Self::now();
        let context = PluginRuntimeContext {
            id: id.clone(),
            plugin_id: registration.plugin_id,
            command_id: registration.command_id,
            plugin_name: registration.plugin_name,
            title: registration.title,
            entry_url: registration.entry_url,
            host_window_label: registration.host_window_label,
            detached_window_label: registration.detached_window_label,
            webview_label: webview_label.clone(),
            placement: registration.placement,
            bounds: registration.bounds,
            permissions: registration.permissions,
            trusted: registration.trusted,
            policy: registration.policy,
            lifecycle: PluginRuntimeLifecycle::Created,
            pending_enter: false,
            entered: false,
            created_at: now.clone(),
            updated_at: now,
        };

        self.runtime_ids_by_webview_label
            .insert(webview_label, id.clone());
        // 无论 policy，均写入索引
        self.runtime_ids_by_plugin_command
            .entry((context.plugin_id.clone(), context.command_id.clone()))
            .or_default()
            .insert(id.clone());
        self.runtimes_by_id.insert(id, context.clone());
        Ok(context)
    }

    pub fn runtime(&self, id: &str) -> Option<PluginRuntimeContext> {
        self.runtimes_by_id.get(id).cloned()
    }

    pub fn runtime_for_webview_label(&self, webview_label: &str) -> Option<PluginRuntimeContext> {
        let id = self.runtime_ids_by_webview_label.get(webview_label)?;
        self.runtime(id)
    }

    pub fn runtime_for_window_label(&self, window_label: &str) -> Option<PluginRuntimeContext> {
        self.runtimes_by_id
            .values()
            .find(|context| context.host_window_label == window_label)
            .cloned()
    }

    /// 取第一个匹配的运行时（Singleton 场景，set 长度 ≤ 1）。
    pub fn runtime_for_plugin_command(
        &self,
        plugin_id: &str,
        command_id: &str,
    ) -> Option<PluginRuntimeContext> {
        let ids = self
            .runtime_ids_by_plugin_command
            .get(&(plugin_id.to_string(), command_id.to_string()))?;
        ids.first().and_then(|id| self.runtime(id))
    }

    /// 取所有匹配的运行时（多实例管理、批量操作、卸载清理用）。
    #[allow(dead_code)]
    pub fn runtimes_for_plugin_command(
        &self,
        plugin_id: &str,
        command_id: &str,
    ) -> Vec<PluginRuntimeContext> {
        self.runtime_ids_by_plugin_command
            .get(&(plugin_id.to_string(), command_id.to_string()))
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.runtimes_by_id.get(id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn mark_lifecycle(
        &mut self,
        id: &str,
        lifecycle: PluginRuntimeLifecycle,
    ) -> Option<PluginRuntimeContext> {
        self.update_by_id(id, |context| {
            context.lifecycle = lifecycle;
        })
    }

    pub fn mark_ready(&mut self, id: &str) -> Option<PluginRuntimeContext> {
        self.update_by_id(id, |context| {
            context.lifecycle = PluginRuntimeLifecycle::Ready;
            if context.pending_enter {
                context.lifecycle = PluginRuntimeLifecycle::Active;
                context.pending_enter = false;
                context.entered = true;
            }
        })
    }

    pub fn mark_focus_enter(&mut self, id: &str) -> Option<PluginRuntimeContext> {
        self.update_by_id(id, |context| {
            if matches!(
                context.lifecycle,
                PluginRuntimeLifecycle::Ready | PluginRuntimeLifecycle::Active
            ) {
                context.lifecycle = PluginRuntimeLifecycle::Active;
                context.pending_enter = false;
                context.entered = true;
            } else {
                context.pending_enter = true;
            }
        })
    }

    pub fn mark_leave(&mut self, id: &str) -> Option<PluginRuntimeContext> {
        self.update_by_id(id, |context| {
            context.entered = false;
            context.pending_enter = false;
            if context.lifecycle != PluginRuntimeLifecycle::Closed {
                context.lifecycle = PluginRuntimeLifecycle::Ready;
            }
        })
    }

    pub fn move_to_host(
        &mut self,
        id: &str,
        host_window_label: String,
        placement: PluginRuntimePlacement,
        bounds: Option<PluginRuntimeBounds>,
    ) -> Option<PluginRuntimeContext> {
        self.update_by_id(id, |context| {
            context.host_window_label = host_window_label;
            context.placement = placement;
            context.bounds = bounds;
        })
    }

    pub fn mark_bounds(
        &mut self,
        id: &str,
        bounds: Option<PluginRuntimeBounds>,
    ) -> Option<PluginRuntimeContext> {
        self.update_by_id(id, |context| {
            context.bounds = bounds;
        })
    }

    pub fn mark_title(&mut self, id: &str, title: String) -> Option<PluginRuntimeContext> {
        self.update_by_id(id, |context| {
            context.title = title;
        })
    }

    pub fn mark_detached_window(
        &mut self,
        id: &str,
        detached_window_label: Option<String>,
    ) -> Option<PluginRuntimeContext> {
        self.update_by_id(id, |context| {
            context.detached_window_label = detached_window_label;
        })
    }

    pub fn remove(&mut self, target: &str) -> Option<PluginRuntimeContext> {
        let id = if self.runtimes_by_id.contains_key(target) {
            target.to_string()
        } else if let Some(id) = self.runtime_ids_by_webview_label.get(target) {
            id.clone()
        } else {
            self.runtimes_by_id.iter().find_map(|(id, context)| {
                (context.host_window_label == target
                    || context.detached_window_label.as_deref() == Some(target))
                .then(|| id.clone())
            })?
        };

        let context = self.runtimes_by_id.remove(&id)?;
        self.runtime_ids_by_webview_label
            .remove(&context.webview_label);
        // 从 BTreeSet 中移除该 runtime_id，set 为空时清理 key
        if let Some(ids) = self
            .runtime_ids_by_plugin_command
            .get_mut(&(context.plugin_id.clone(), context.command_id.clone()))
        {
            ids.remove(&context.id);
            if ids.is_empty() {
                self.runtime_ids_by_plugin_command
                    .remove(&(context.plugin_id.clone(), context.command_id.clone()));
            }
        }
        Some(context)
    }

    fn update_by_id(
        &mut self,
        id: &str,
        update: impl FnOnce(&mut PluginRuntimeContext),
    ) -> Option<PluginRuntimeContext> {
        let context = self.runtimes_by_id.get_mut(id)?;
        update(context);
        context.updated_at = Self::now();
        Some(context.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::windowing::labels::MAIN_WINDOW_LABEL;

    use super::*;

    fn registration() -> PluginRuntimeRegistration {
        PluginRuntimeRegistration {
            plugin_id: "dev.litools.test".to_string(),
            command_id: "hello".to_string(),
            plugin_name: "Test".to_string(),
            title: "Hello".to_string(),
            entry_url: "litools-plugin://dev.litools.test/dist/index.html".to_string(),
            host_window_label: MAIN_WINDOW_LABEL.to_string(),
            detached_window_label: None,
            placement: PluginRuntimePlacement::Docked,
            bounds: None,
            permissions: vec!["ui:window".to_string()],
            trusted: false,
            policy: RuntimePolicy::Singleton,
        }
    }

    fn multi_registration() -> PluginRuntimeRegistration {
        let mut reg = registration();
        reg.policy = RuntimePolicy::MultiInstance;
        reg
    }

    #[test]
    fn registers_and_finds_runtime() {
        let mut registry = PluginRuntimeRegistry::default();
        let context = registry
            .register_runtime(
                registration(),
                "runtime_000001".to_string(),
                "plugin-runtime-runtime_000001".to_string(),
            )
            .expect("runtime registered");

        assert_eq!(context.lifecycle, PluginRuntimeLifecycle::Created);
        assert_eq!(context.placement, PluginRuntimePlacement::Docked);
        assert_eq!(context.policy, RuntimePolicy::Singleton);
        assert_eq!(
            registry.runtime_for_webview_label("plugin-runtime-runtime_000001"),
            Some(context.clone())
        );
        assert_eq!(
            registry.runtime_for_plugin_command("dev.litools.test", "hello"),
            Some(context)
        );
    }

    #[test]
    fn rejects_duplicate_singleton_runtime() {
        let mut registry = PluginRuntimeRegistry::default();
        registry
            .register_runtime(
                registration(),
                "runtime_000001".to_string(),
                "plugin-runtime-runtime_000001".to_string(),
            )
            .expect("runtime registered");

        assert!(
            registry
                .register_runtime(
                    registration(),
                    "runtime_000002".to_string(),
                    "plugin-runtime-runtime_000002".to_string(),
                )
                .is_err()
        );
    }

    #[test]
    fn allows_multi_instance_runtimes() {
        let mut registry = PluginRuntimeRegistry::default();
        registry
            .register_runtime(
                multi_registration(),
                "runtime_000001".to_string(),
                "plugin-runtime-runtime_000001".to_string(),
            )
            .expect("first runtime registered");

        let second = registry
            .register_runtime(
                multi_registration(),
                "runtime_000002".to_string(),
                "plugin-runtime-runtime_000002".to_string(),
            )
            .expect("second runtime registered");

        assert_eq!(second.policy, RuntimePolicy::MultiInstance);
        let all = registry.runtimes_for_plugin_command("dev.litools.test", "hello");
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn moves_runtime_from_docked_to_detached() {
        let mut registry = PluginRuntimeRegistry::default();
        registry
            .register_runtime(
                registration(),
                "runtime_000001".to_string(),
                "plugin-runtime-runtime_000001".to_string(),
            )
            .expect("runtime registered");

        let bounds = PluginRuntimeBounds {
            x: 0.0,
            y: 68.0,
            width: 820.0,
            height: 492.0,
        };
        let context = registry
            .move_to_host(
                "runtime_000001",
                "plugin-runtime-window-runtime_000001".to_string(),
                PluginRuntimePlacement::Detached,
                Some(bounds),
            )
            .expect("runtime moved");
        let context = registry
            .mark_detached_window(
                &context.id,
                Some("plugin-runtime-window-runtime_000001".to_string()),
            )
            .expect("detached window marked");

        assert_eq!(context.placement, PluginRuntimePlacement::Detached);
        assert_eq!(context.bounds, Some(bounds));
        assert_eq!(
            registry.runtime_for_webview_label("plugin-runtime-runtime_000001"),
            Some(context.clone())
        );
        assert_eq!(
            registry.runtime_for_plugin_command("dev.litools.test", "hello"),
            Some(context)
        );
    }

    #[test]
    fn transitions_lifecycle() {
        let mut registry = PluginRuntimeRegistry::default();
        registry
            .register_runtime(
                registration(),
                "runtime_000001".to_string(),
                "plugin-runtime-runtime_000001".to_string(),
            )
            .expect("runtime registered");

        let context = registry
            .mark_focus_enter("runtime_000001")
            .expect("runtime updated");
        assert_eq!(context.lifecycle, PluginRuntimeLifecycle::Created);
        assert!(context.pending_enter);
        assert!(!context.entered);

        let context = registry
            .mark_ready("runtime_000001")
            .expect("runtime ready");
        assert_eq!(context.lifecycle, PluginRuntimeLifecycle::Active);
        assert!(!context.pending_enter);
        assert!(context.entered);
    }

    #[test]
    fn removes_runtime_indexes() {
        let mut registry = PluginRuntimeRegistry::default();
        registry
            .register_runtime(
                registration(),
                "runtime_000001".to_string(),
                "plugin-runtime-runtime_000001".to_string(),
            )
            .expect("runtime registered");

        assert!(registry.remove(MAIN_WINDOW_LABEL).is_some());
        assert!(registry.runtime("runtime_000001").is_none());
        assert!(
            registry
                .runtime_for_webview_label("plugin-runtime-runtime_000001")
                .is_none()
        );
    }
}
