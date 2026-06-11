// ---- re-export from core (single source of truth) ----

export type {
    AppSettings,
    AppWatcherStatus,
    CommandEffect,
    CommandExecution,
    DiagnosticsResponse,
    IconCacheSummary,
    IndexStatus,
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
    ReloadIndexSummary,
    SearchResult,
    SearchResultAction,
    SearchResultMatches,
    ShortcutStatus,
    SurfaceMetadata,
    UsageEvent,
    ViewProvider,
    WindowHostKind,
} from '@litools/plugin-core';

// ---- bridge-only types (not in core) ----

export type PluginViewBounds = {
    x: number;
    y: number;
    width: number;
    height: number;
};
