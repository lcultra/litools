import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, CommandExecution, DiagnosticsResponse, IndexStatus, LauncherPanelResponse, SearchResult } from './types';

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

export function hideMainWindow(): Promise<void> {
    return invoke<void>('hide_main_window');
}

export function showMainWindow(): Promise<void> {
    return invoke<void>('show_main_window');
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
