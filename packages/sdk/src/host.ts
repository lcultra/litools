import { invoke } from '@tauri-apps/api/core';
import type { PluginSummary, PluginViewDescriptor, PluginViewInfo } from './types';

const CORE = 'plugin:litools-core';

function invokeCore<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(`${CORE}|${cmd}`, args);
}

// ── host API（仅 trusted 插件可用） ──
// 通过 `import { host } from '@litools/sdk'` 访问，如 host.plugins.list()

export const plugins = {
  list(): Promise<PluginSummary[]> {
    return invokeCore('list_plugins');
  },
  toggle(pluginId: string, enabled: boolean): Promise<PluginSummary> {
    return invokeCore('toggle_plugin', { pluginId, enabled });
  },
  install(filePath: string): Promise<PluginSummary> {
    return invokeCore('install_plugin', { filePath });
  },
  uninstall(pluginId: string): Promise<void> {
    return invokeCore('uninstall_plugin', { pluginId });
  },
  /** 获取插件视图描述符 */
  getViewDescriptor(pluginId: string, commandId: string): Promise<PluginViewDescriptor> {
    return invokeCore('get_plugin_view_descriptor', { pluginId, commandId });
  },
  /** 打开插件视图（dock） */
  openView(pluginId: string, commandId: string): Promise<PluginViewInfo> {
    return invokeCore('open_plugin_view', { pluginId, commandId });
  },
  /** 隐藏插件视图 */
  hideView(pluginId: string, commandId: string): Promise<PluginViewInfo> {
    return invokeCore('hide_plugin_view', { pluginId, commandId });
  },
  /** 按 runtimeId 隐藏插件视图 */
  hideViewById(runtimeId: string): Promise<PluginViewInfo> {
    return invokeCore('hide_plugin_view_by_id', { runtimeId });
  },
  /** 分离插件视图到独立窗口 */
  detachView(pluginId: string, commandId: string): Promise<PluginViewInfo> {
    return invokeCore('detach_plugin_view', { pluginId, commandId });
  },
  /** 按 runtimeId 分离插件视图到独立窗口 */
  detachViewById(runtimeId: string): Promise<PluginViewInfo> {
    return invokeCore('detach_plugin_view_by_id', { runtimeId });
  },
  /** 关闭插件视图 */
  closeView(pluginId: string, commandId: string): Promise<void> {
    return invokeCore('close_plugin_view', { pluginId, commandId });
  },
  /** 按 runtimeId 关闭插件视图 */
  closeViewById(runtimeId: string): Promise<void> {
    return invokeCore('close_plugin_view_by_id', { runtimeId });
  },
  /** 获取插件视图运行时信息 */
  getViewInfo(runtimeId: string): Promise<PluginViewInfo> {
    return invokeCore('get_plugin_view_info', { runtimeId });
  },
};

export const devtools = {
  open(runtimeId: string): Promise<void> {
    return invokeCore('open_plugin_devtools', { runtimeId });
  },
};

export const index = {
  reload(): Promise<import('./types').IndexStatus> {
    return invokeCore('reload_index');
  },
};
