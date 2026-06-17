// 宿主内部 bridge —— 直接 invoke Tauri 命令，不依赖任何 npm 包
// 这些是主窗口独占的特权操作，插件不能调用
import { invoke } from '@tauri-apps/api/core';

const CORE = 'plugin:litools-core';
function invokeCore<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    return invoke<T>(`${CORE}|${cmd}`, args);
}

// ── 搜索 ──
export const search = (query: string) => invokeCore<import('@litools/sdk').SearchResult[]>('search', { query });
export const launcherPanel = (query: string) => invokeCore<import('@litools/sdk').LauncherPanelResponse>('launcher_panel', { query });
export const executeResult = (resultId: string, actionId: string, provider: string) =>
    invokeCore<import('@litools/sdk').CommandExecution>('execute_result', { resultId, actionId, provider });
export const revealInFileManager = (resultId: string) => invokeCore<void>('reveal_in_file_manager', { resultId });

// ── 固定 ──
export const pinResult = (resultId: string) => invokeCore<void>('pin_result', { resultId });
export const unpinResult = (resultId: string) => invokeCore<void>('unpin_result', { resultId });
export const reorderPinnedResults = (resultIds: string[]) => invokeCore<void>('reorder_pinned_results', { resultIds });

// ── 窗口控制 ──
export const showMainWindow = () => invokeCore<void>('show_main_window');
export const hideMainWindow = () => invokeCore<void>('hide_main_window');
export const focusMainWindow = () => invokeCore<void>('focus_main_window');
export const resizeMainWindowHeight = (height: number) => invokeCore<void>('resize_main_window_height', { height });
export const getBaseInfo = () => invokeCore<import('@litools/sdk').BaseInfo>('get_base_info');
export const startWindowDragging = () => invokeCore<void>('start_window_dragging');

// ── Surface 管理 ──
export const listSurfaces = () => invokeCore<import('@litools/sdk').SurfaceMetadata[]>('list_windows');
export const getCurrentSurfaceMetadata = () => invokeCore<import('@litools/sdk').SurfaceMetadata | null>('get_current_window_metadata');
export const detachRoute = (route: string) => invokeCore<import('@litools/sdk').SurfaceMetadata>('detach_route', { route });
export const updateSurfaceRoute = (route: string) => invokeCore<import('@litools/sdk').SurfaceMetadata>('update_surface_route', { route });
export const hideSurface = (target?: string | { main: boolean }) => invokeCore<void>('hide_window', { target });
export const focusSurface = (target?: string | { main: boolean }) => invokeCore<void>('focus_window', { target });
export const destroySurface = (target: string) => invokeCore<void>('destroy_window', { target });

// ── 插件视图 ──
export const openPluginView = (pluginId: string, commandId: string) => invokeCore<import('@litools/sdk').PluginViewInfo>('open_plugin_view', { pluginId, commandId });
export const hidePluginView = (pluginId: string, commandId: string) => invokeCore<import('@litools/sdk').PluginViewInfo>('hide_plugin_view', { pluginId, commandId });
export const hidePluginViewById = (runtimeId: string) => invokeCore<import('@litools/sdk').PluginViewInfo>('hide_plugin_view_by_id', { runtimeId });
export const detachPluginView = (pluginId: string, commandId: string) => invokeCore<import('@litools/sdk').PluginViewInfo>('detach_plugin_view', { pluginId, commandId });
export const detachPluginViewById = (runtimeId: string) => invokeCore<import('@litools/sdk').PluginViewInfo>('detach_plugin_view_by_id', { runtimeId });
export const closePluginView = (pluginId: string, commandId: string) => invokeCore<void>('close_plugin_view', { pluginId, commandId });
export const closePluginViewById = (runtimeId: string) => invokeCore<void>('close_plugin_view_by_id', { runtimeId });
export const getPluginViewInfo = (runtimeId: string) => invokeCore<import('@litools/sdk').PluginViewInfo>('get_plugin_view_info', { runtimeId });
export const getPluginViewDescriptor = (pluginId: string, commandId: string) =>
    invokeCore<import('@litools/sdk').PluginViewDescriptor>('get_plugin_view_descriptor', { pluginId, commandId });
export const openPluginDevtools = (runtimeId: string) => invokeCore<void>('open_plugin_devtools', { runtimeId });

// ── 插件管理 ──
export const listPlugins = () => invokeCore<import('@litools/sdk').PluginSummary[]>('list_plugins');
export const togglePlugin = (pluginId: string, enabled: boolean) => invokeCore<import('@litools/sdk').PluginSummary>('toggle_plugin', { pluginId, enabled });
export const installPlugin = (filePath: string) => invokeCore<import('@litools/sdk').PluginSummary>('install_plugin', { filePath });
export const uninstallPlugin = (pluginId: string) => invokeCore<void>('uninstall_plugin', { pluginId });

// ── 设置 & 索引 ──
export const getSettings = () => invokeCore<import('@litools/sdk').AppSettings>('get_settings');
export const updateSettings = (settings: import('@litools/sdk').AppSettings) => invokeCore<import('@litools/sdk').AppSettings>('update_settings', { settings });
export const reloadIndex = () => invokeCore<import('@litools/sdk').IndexStatus>('reload_index');
