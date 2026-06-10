import { invoke } from '@tauri-apps/api/core';
import type { AppRoutePath } from '../shared/routes';
import type {
    AppSettings,
    CommandExecution,
    IndexStatus,
    LauncherPanelResponse,
    PluginSummary,
    PluginViewDescriptor,
    PluginViewInfo,
    SearchResult,
    SurfaceMetadata,
    WindowHostKind,
} from './types';

export function search(query: string): Promise<SearchResult[]> {
    return invoke<SearchResult[]>('search', { query });
}

export function launcherPanel(query: string): Promise<LauncherPanelResponse> {
    return invoke<LauncherPanelResponse>('launcher_panel', { query });
}

export function pinResult(resultId: string): Promise<void> {
    return invoke<void>('pin_result', { resultId });
}

export function unpinResult(resultId: string): Promise<void> {
    return invoke<void>('unpin_result', { resultId });
}

export function reorderPinnedResults(resultIds: string[]): Promise<void> {
    return invoke<void>('reorder_pinned_results', { resultIds });
}

export function executeResult(resultId: string, actionId: string): Promise<CommandExecution> {
    return invoke<CommandExecution>('execute_result', { resultId, actionId });
}

export function detachRoute(route: AppRoutePath): Promise<SurfaceMetadata> {
    return invoke<SurfaceMetadata>('detach_route', { route });
}

export function updateSurfaceRoute(route: AppRoutePath): Promise<SurfaceMetadata> {
    return invoke<SurfaceMetadata>('update_surface_route', { route });
}

export function hideSurface(target?: WindowHostKind | string): Promise<void> {
    return invoke<void>('hide_window', { target });
}

export function focusSurface(target?: WindowHostKind | string): Promise<void> {
    return invoke<void>('focus_window', { target });
}

export function listSurfaces(): Promise<SurfaceMetadata[]> {
    return invoke<SurfaceMetadata[]>('list_windows');
}

export function getCurrentSurfaceMetadata(): Promise<SurfaceMetadata | null> {
    return invoke<SurfaceMetadata | null>('get_current_window_metadata');
}

export function destroySurface(target: string): Promise<void> {
    return invoke<void>('destroy_window', { target });
}

export function startWindowDragging(): Promise<void> {
    return invoke<void>('start_window_dragging');
}

export function hideMainWindow(): Promise<void> {
    return invoke<void>('hide_main_window');
}

export function showMainWindow(): Promise<void> {
    return invoke<void>('show_main_window');
}

export function focusMainWindow(): Promise<void> {
    return invoke<void>('focus_main_window');
}

export function resizeMainWindowHeight(height: number): Promise<void> {
    return invoke<void>('resize_main_window_height', { height });
}

export function reloadIndex(): Promise<IndexStatus> {
    return invoke<IndexStatus>('reload_index');
}

export function getSettings(): Promise<AppSettings> {
    return invoke<AppSettings>('get_settings');
}

export function updateSettings(settings: AppSettings): Promise<AppSettings> {
    return invoke<AppSettings>('update_settings', { settings });
}

export function listPlugins(): Promise<PluginSummary[]> {
    return invoke<PluginSummary[]>('list_plugins');
}

export function getPluginViewDescriptor(pluginId: string, commandId: string): Promise<PluginViewDescriptor> {
    return invoke<PluginViewDescriptor>('get_plugin_view_descriptor', { pluginId, commandId });
}

export function openPluginView(pluginId: string, commandId: string): Promise<PluginViewInfo> {
    return invoke<PluginViewInfo>('open_plugin_view', { pluginId, commandId });
}

export function hidePluginView(pluginId: string, commandId: string): Promise<PluginViewInfo> {
    return invoke<PluginViewInfo>('hide_plugin_view', { pluginId, commandId });
}

export function detachPluginView(pluginId: string, commandId: string): Promise<PluginViewInfo> {
    return invoke<PluginViewInfo>('detach_plugin_view', { pluginId, commandId });
}

export function closePluginView(pluginId: string, commandId: string): Promise<void> {
    return invoke<void>('close_plugin_view', { pluginId, commandId });
}

export function closePluginViewById(runtimeId: string): Promise<void> {
    return invoke<void>('close_plugin_view_by_id', { runtimeId });
}

export function getPluginViewInfo(runtimeId: string): Promise<PluginViewInfo> {
    return invoke<PluginViewInfo>('get_plugin_view_info', { runtimeId });
}
