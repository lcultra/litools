// ---- re-export from core (single source of truth) ----

import type { AppSettings } from '@litools/plugin-core';

export type {
    AppSettings,
    CommandEffect,
    CommandExecution,
    LauncherItem,
    LauncherPanelResponse,
    LauncherSection,
    MatchRange,
    PluginCommandMode,
    PluginCommandSummary,
    PluginSource,
    PluginSummary,
    PluginViewDescriptor,
    PluginViewInfo,
    PluginViewLifecycle,
    PluginViewState,
    SearchResult,
    SearchResultAction,
    SearchResultMatches,
    SurfaceMetadata,
    ViewProvider,
    WindowHostKind,
} from '@litools/plugin-core';

// ---- bridge‑only types (not in core) ----

export type PluginViewBounds = {
    x: number;
    y: number;
    width: number;
    height: number;
};

export type SurfaceLifecycle = 'active' | 'hidden' | 'destroyed';

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

export type IndexStatus = {
    running: boolean;
    pending: boolean;
    lastTrigger?: string | null;
    lastError?: string | null;
    lastSummary?: unknown;
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
