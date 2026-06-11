import { invoke } from '@tauri-apps/api/core';

const CORE = 'plugin:litools-core';

function invokeCore<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    return invoke<T>(`${CORE}|${cmd}`, args);
}

// ---- types (single source of truth) ----

export type ThemeValue = 'system' | 'light' | 'dark';

export type SearchResultAction = { id: string; label: string };
export type MatchRange = { start: number; end: number };
export type SearchResultMatches = { title: MatchRange[]; subtitle: MatchRange[] };
export type SearchResult = {
    id: string; title: string; subtitle?: string | null; iconUri?: string | null;
    provider: string; score: number; matches?: SearchResultMatches; actions: SearchResultAction[];
};
export type CommandEffect = 'none' | 'openLogsDirectory' | 'openDataDirectory'
    | { openPluginView: { plugin_id: string; command_id: string; route: string } }
    | 'reloadIndex' | 'quitApp' | 'toggleTheme';
export type CommandExecution = { resultId: string; actionId: string; message: string; effect: CommandEffect };
export type PluginCommandMode = 'instant' | 'view' | 'searchProvider';
export type PluginSource = 'bundled' | 'user';
export type PluginCommandSummary = { id: string; title: string; subtitle?: string | null; keywords: string[]; mode: PluginCommandMode };
export type PluginSummary = {
    id: string; name: string; version: string; description?: string | null; author?: string | null;
    icon: string; enabled: boolean; trusted: boolean; source: PluginSource; path: string;
    permissions: string[]; commands: PluginCommandSummary[];
};
export type PluginViewDescriptor = {
    pluginId: string; commandId: string; pluginName: string; title: string;
    entryUrl: string; icon: string; permissions: string[]; dev: boolean;
};
export type PluginViewPlacement = 'docked' | 'detached';
export type PluginViewLifecycle = 'created' | 'ready' | 'active' | 'closed' | 'failed';
export type PluginViewInfo = {
    runtimeId: string; pluginId: string; commandId: string; pluginName: string; title: string;
    surfaceId: string; hostKind?: string | null;
    lifecycle: PluginViewLifecycle; permissions: string[];
};
export type PluginViewState = {
    pluginId: string; commandId: string; pluginName: string; title: string;
    lifecycle: PluginViewLifecycle;
    runtimeId: string | null; dev: boolean;
};
export type ViewProvider = 'core' | { plugin: { pluginId: string } };
export type WindowHostKind = 'main' | 'detached';
export type SurfaceMetadata = {
    id: string; webviewLabel: string; viewId: string; provider: ViewProvider; route: string;
    title: string; hostWindowLabel: string; hostKind: WindowHostKind;
    lifecycle: 'active' | 'hidden' | 'destroyed'; focused: boolean; createdAt: string; updatedAt: string;
};
export type LauncherItem = { result: SearchResult; isPinned: boolean };
export type LauncherSection = { id: string; title: string; items: LauncherItem[] };
export type LauncherPanelResponse = { sections: LauncherSection[] };
export type IndexStatus = { running: boolean; pending: boolean; lastTrigger?: string | null; lastError?: string | null; lastSummary?: unknown };
export type AppSettings = {
    theme: ThemeValue;
    palette: { global_hotkey: string; show_recent: boolean; show_pinned: boolean };
    search: { enabled_providers: string[] };
    window: { hide_on_blur: boolean; close_to_tray: boolean; center_on_show: boolean };
};

// ---- diagnostics types (moved from bridge/types.ts) ----

export type ReloadIndexSummary = {
    trigger: string;
    startedAt: string;
    finishedAt: string;
    durationMs: number;
    commandsUpserted: number;
    appsDiscovered: number;
    appsUpserted: number;
    appsRemoved: number;
    success: boolean;
    error?: string | null;
};

export type UsageEvent = {
    target_type: string;
    target_id: string;
    query?: string | null;
    selected_at: string;
};

export type AppWatcherStatus = {
    platform: string;
    enabled: boolean;
    status: string;
    watchedPaths: string[];
    error?: string | null;
};

export type IconCacheSummary = {
    fileCount: number;
    totalBytes: number;
    maxFiles: number;
    maxBytes: number;
    lastPrunedAt?: string | null;
    lastPrunedFiles: number;
    error?: string | null;
};

export type ShortcutStatus = {
    accelerator: string;
    registered: boolean;
    error?: string | null;
};

export type DiagnosticsResponse = {
    app_version: string;
    app_data_dir: string;
    platform: string;
    plugin_count: number;
    command_count: number;
    app_count: number;
    index_status: IndexStatus;
    last_persisted_index_status?: ReloadIndexSummary | null;
    app_watcher: AppWatcherStatus;
    icon_cache: IconCacheSummary;
    recent_usage_count: number;
    recent_usage: UsageEvent[];
    settings: AppSettings;
    shortcut: ShortcutStatus;
};

// ---- commands ----

export function search(query: string): Promise<SearchResult[]> { return invokeCore('search', { query }); }
export function launcherPanel(query: string): Promise<LauncherPanelResponse> { return invokeCore('launcher_panel', { query }); }
export function pinResult(resultId: string): Promise<void> { return invokeCore('pin_result', { resultId }); }
export function unpinResult(resultId: string): Promise<void> { return invokeCore('unpin_result', { resultId }); }
export function reorderPinnedResults(resultIds: string[]): Promise<void> { return invokeCore('reorder_pinned_results', { resultIds }); }
export function executeResult(resultId: string, actionId: string): Promise<CommandExecution> { return invokeCore('execute_result', { resultId, actionId }); }
export function detachRoute(route: string): Promise<SurfaceMetadata> { return invokeCore('detach_route', { route }); }
export function updateSurfaceRoute(route: string): Promise<SurfaceMetadata> { return invokeCore('update_surface_route', { route }); }
export function hideSurface(target?: WindowHostKind | string): Promise<void> { return invokeCore('hide_window', { target }); }
export function focusSurface(target?: WindowHostKind | string): Promise<void> { return invokeCore('focus_window', { target }); }
export function listSurfaces(): Promise<SurfaceMetadata[]> { return invokeCore('list_windows'); }
export function getCurrentSurfaceMetadata(): Promise<SurfaceMetadata | null> { return invokeCore('get_current_window_metadata'); }
export function destroySurface(target: string): Promise<void> { return invokeCore('destroy_window', { target }); }
export function startWindowDragging(): Promise<void> { return invokeCore('start_window_dragging'); }
export function hideMainWindow(): Promise<void> { return invokeCore('hide_main_window'); }
export function showMainWindow(): Promise<void> { return invokeCore('show_main_window'); }
export function focusMainWindow(): Promise<void> { return invokeCore('focus_main_window'); }
export function resizeMainWindowHeight(height: number): Promise<void> { return invokeCore('resize_main_window_height', { height }); }
export function revealInFileManager(resultId: string): Promise<void> { return invokeCore('reveal_in_file_manager', { resultId }); }
export function reloadIndex(): Promise<IndexStatus> { return invokeCore('reload_index'); }
export function getSettings(): Promise<AppSettings> { return invokeCore('get_settings'); }
export function updateSettings(settings: AppSettings): Promise<AppSettings> { return invokeCore('update_settings', { settings }); }
export function listPlugins(): Promise<PluginSummary[]> { return invokeCore('list_plugins'); }
export function getPluginViewDescriptor(pluginId: string, commandId: string): Promise<PluginViewDescriptor> { return invokeCore('get_plugin_view_descriptor', { pluginId, commandId }); }
export function openPluginView(pluginId: string, commandId: string): Promise<PluginViewInfo> { return invokeCore('open_plugin_view', { pluginId, commandId }); }
export function hidePluginView(pluginId: string, commandId: string): Promise<PluginViewInfo> { return invokeCore('hide_plugin_view', { pluginId, commandId }); }
export function detachPluginView(pluginId: string, commandId: string): Promise<PluginViewInfo> { return invokeCore('detach_plugin_view', { pluginId, commandId }); }
export function closePluginView(pluginId: string, commandId: string): Promise<void> { return invokeCore('close_plugin_view', { pluginId, commandId }); }
export function closePluginViewById(runtimeId: string): Promise<void> { return invokeCore('close_plugin_view_by_id', { runtimeId }); }
export function getPluginViewInfo(runtimeId: string): Promise<PluginViewInfo> { return invokeCore('get_plugin_view_info', { runtimeId }); }
export function openPluginDevtools(runtimeId: string): Promise<void> { return invokeCore('open_plugin_devtools', { runtimeId }); }
export function togglePlugin(pluginId: string, enabled: boolean): Promise<void> { return invokeCore('toggle_plugin', { pluginId, enabled }); }
export function installPlugin(filePath: string): Promise<PluginSummary> { return invokeCore('install_plugin', { filePath }); }
export function uninstallPlugin(pluginId: string): Promise<void> { return invokeCore('uninstall_plugin', { pluginId }); }
