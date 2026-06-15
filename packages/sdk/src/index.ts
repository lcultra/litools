import { invoke } from '@tauri-apps/api/core';

// ---- helpers ----

const SDK = 'plugin:litools-sdk';

function invokeSdk<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(`${SDK}|${cmd}`, args);
}

// ---- runtime ----

export function ready(): Promise<PluginRuntimeInfo> {
  return invokeSdk('sdk_runtime_ready');
}

export function getInfo(): Promise<PluginRuntimeInfo> {
  return invokeSdk('sdk_runtime_get_info');
}

// ---- permissions ----

export function queryPermission(permission: string): Promise<PermissionQueryResult> {
  return invokeSdk('sdk_permissions_query', { permission });
}

// ---- lifecycle ----

export function onEnter(cb: () => void): () => void {
  const listeners = (window as any).__litoolsLifecycleListeners;
  if (!listeners) throw new Error('litools runtime not available');
  listeners.enter.add(cb);
  return () => listeners.enter.delete(cb);
}

export function onLeave(cb: () => void): () => void {
  const listeners = (window as any).__litoolsLifecycleListeners;
  if (!listeners) throw new Error('litools runtime not available');
  listeners.leave.add(cb);
  return () => listeners.leave.delete(cb);
}

// ---- ui ----

export function close(): Promise<void> {
  return invokeSdk('sdk_ui_close');
}

export function setTitle(title: string): Promise<void> {
  return invokeSdk('sdk_ui_set_title', { title });
}

export function toast(message: string, options?: Record<string, unknown>): Promise<void> {
  return invokeSdk('sdk_ui_toast', { message, options });
}

// ---- storage ----

export function storageGet<T = unknown>(key: string): Promise<T | null> {
  return invokeSdk<T>('sdk_storage_get', { key });
}

export function storageSet(key: string, value: unknown): Promise<void> {
  return invokeSdk('sdk_storage_set', { key, value });
}

export function storageRemove(key: string): Promise<void> {
  return invokeSdk('sdk_storage_remove', { key });
}

export function storageClear(): Promise<void> {
  return invokeSdk('sdk_storage_clear');
}

// ---- settings ----

export function settingsGet(): Promise<AppSettings> {
  return invokeSdk('sdk_settings_get');
}

export function settingsUpdate(settings: AppSettings): Promise<AppSettings> {
  return invokeSdk('sdk_settings_update', { settings });
}

// ---- diagnostics / plugins ----

export function diagnosticsGet(): Promise<DiagnosticsResponse> {
  return invokeSdk('sdk_diagnostics_get');
}

export function pluginsList(): Promise<PluginSummary[]> {
  return invokeSdk('sdk_plugins_list');
}

// ---- types ----

export type PluginPermission =
  | 'clipboard:read' | 'clipboard:write'
  | 'diagnostics:read' | 'files:open' | 'files:reveal'
  | 'plugins:list' | 'settings:read' | 'settings:write'
  | 'storage:plugin' | 'ui:toast' | 'ui:window';

export type PluginManifest = {
  id: string;
  name: string;
  version: string;
  entry: string;
  description?: string;
  author?: string;
  icon: string;
  commands?: PluginCommand[];
  permissions?: PluginPermission[];
  development?: { main: string };
};

export type PluginCommand = {
  id: string;
  title: string;
  subtitle?: string;
  keywords?: string[];
  mode: 'instant' | 'view' | 'searchProvider';
};

export type PluginRuntimeLifecycle = 'created' | 'ready' | 'active' | 'closed' | 'failed';
export type PluginRuntimePlacement = 'docked' | 'detached';

export type PluginRuntimeInfo = {
  runtimeId: string;
  pluginId: string;
  commandId: string;
  pluginName: string;
  title: string;
  hostWindowLabel: string;
  detachedWindowLabel?: string | null;
  webviewLabel: string;
  placement: PluginRuntimePlacement;
  bounds?: { x: number; y: number; width: number; height: number } | null;
  lifecycle: PluginRuntimeLifecycle;
  permissions: string[];
};

export type PermissionQueryResult = {
  permission: string;
  state: 'granted' | 'denied';
};

export type PluginSummary = {
  id: string;
  name: string;
  version: string;
  description?: string | null;
  author?: string | null;
  icon: string;
  enabled: boolean;
  trusted: boolean;
  source: 'bundled' | 'user';
  path: string;
  permissions: string[];
  commands: Array<{
    id: string; title: string; subtitle?: string | null; keywords: string[]; mode: string;
  }>;
};

export type AppSettings = {
  theme: 'system' | 'light' | 'dark';
  palette: { global_hotkey: string; show_recent: boolean; show_pinned: boolean };
  search: { enabled_providers: string[] };
  window: { hide_on_blur: boolean; close_to_tray: boolean; center_on_show: boolean };
};

export type DiagnosticsResponse = {
  app_version: string;
  app_data_dir: string;
  platform: string;
  plugin_count: number;
  command_count: number;
  app_count: number;
  index_status: { running: boolean; pending: boolean; lastTrigger?: string | null; lastError?: string | null; lastSummary?: unknown };
};

// ---- dynamic commands (v2) ----

export type DynamicCommand = {
  id: string;
  title: string;
  subtitle?: string;
  keywords?: string[];
  mode?: 'instant' | 'view' | 'searchProvider';
  executor?: string;
  icon?: string;
  script?: string;
  lifecycle?: 'permanent' | 'session' | 'runtime';
};

// 批量 API
export function addCommands(commands: DynamicCommand[]): Promise<void> {
  return invokeSdk('sdk_commands_add', { commands });
}
export function removeCommands(ids: string[]): Promise<void> {
  return invokeSdk('sdk_commands_remove', { ids });
}
export function replaceCommands(commands: DynamicCommand[]): Promise<void> {
  return invokeSdk('sdk_commands_replace', { commands });
}

// 单条语法糖
export function addCommand(cmd: DynamicCommand): Promise<void> {
  return addCommands([cmd]);
}
export function removeCommand(id: string): Promise<void> {
  return removeCommands([id]);
}
export function updateCommand(id: string, cmd: Partial<DynamicCommand>): Promise<void> {
  return invokeSdk('sdk_commands_update', { id, cmd });
}
