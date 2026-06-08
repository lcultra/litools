import { invoke } from '@tauri-apps/api/core';
import type { AppRoutePath, WindowTarget } from '../views/registry';
import type { AppSettings, CommandExecution, DiagnosticsResponse, IndexStatus, LauncherPanelResponse, ManagedWindowMetadata, SearchResult } from './types';

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

export function detachRoute(route: AppRoutePath): Promise<ManagedWindowMetadata> {
    return invoke<ManagedWindowMetadata>('detach_route', { route });
}

export function updateSurfaceRoute(route: AppRoutePath): Promise<ManagedWindowMetadata> {
    return invoke<ManagedWindowMetadata>('update_surface_route', { route });
}

export function openRoute(route: AppRoutePath, target?: WindowTarget): Promise<void> {
    return invoke<void>('open_route', { route, target });
}

export function hideWindow(target?: WindowTarget | string): Promise<void> {
    return invoke<void>('hide_window', { target });
}

export function focusWindow(target?: WindowTarget | string): Promise<void> {
    return invoke<void>('focus_window', { target });
}

export function listWindows(): Promise<ManagedWindowMetadata[]> {
    return invoke<ManagedWindowMetadata[]>('list_windows');
}

export function getCurrentWindowMetadata(): Promise<ManagedWindowMetadata | null> {
    return invoke<ManagedWindowMetadata | null>('get_current_window_metadata');
}

export function destroyWindow(target: string): Promise<void> {
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

export function openSettings(): Promise<void> {
    return invoke<void>('open_settings');
}

export function focusMainWindow(): Promise<void> {
    return invoke<void>('focus_main_window');
}

export function startDragging(): Promise<void> {
    return invoke<void>('start_dragging');
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

export function getDiagnostics(): Promise<DiagnosticsResponse> {
    return invoke<DiagnosticsResponse>('get_diagnostics');
}
