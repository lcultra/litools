import type { AppRoutePath } from '../shared/routes';
import type { ThemeValue } from '../shared/theme';

export type SearchResultAction = {
    id: string;
    label: string;
};

export type MatchRange = {
    start: number;
    end: number;
};

export type SearchResultMatches = {
    title: MatchRange[];
    subtitle: MatchRange[];
};

export type SearchResult = {
    id: string;
    title: string;
    subtitle?: string | null;
    iconUri?: string | null;
    provider: string;
    score: number;
    matches?: SearchResultMatches;
    actions: SearchResultAction[];
};

export type CommandEffect =
    | 'none'
    | 'openLogsDirectory'
    | 'openDataDirectory'
    | { openPluginView: { pluginId: string; commandId: string; route: AppRoutePath } }
    | 'reloadIndex'
    | 'quitApp'
    | 'toggleTheme';

export type CommandExecution = {
    resultId: string;
    actionId: string;
    message: string;
    effect: CommandEffect;
};

export type PluginCommandMode = 'instant' | 'view' | 'searchProvider';
export type PluginSource = 'bundled' | 'user';

export type PluginCommandSummary = {
    id: string;
    title: string;
    subtitle?: string | null;
    keywords: string[];
    mode: PluginCommandMode;
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
    source: PluginSource;
    path: string;
    permissions: string[];
    commands: PluginCommandSummary[];
};

export type PluginViewDescriptor = {
    pluginId: string;
    commandId: string;
    pluginName: string;
    title: string;
    entryUrl: string;
    permissions: string[];
};

export type PluginViewPlacement = 'docked' | 'detached';
export type PluginViewLifecycle = 'created' | 'ready' | 'active' | 'closed' | 'failed';

export type PluginViewBounds = {
    x: number;
    y: number;
    width: number;
    height: number;
};

export type PluginViewInfo = {
    runtimeId: string;
    pluginId: string;
    commandId: string;
    pluginName: string;
    title: string;
    hostWindowLabel: string;
    detachedWindowLabel?: string | null;
    titlebarWebviewLabel?: string | null;
    webviewLabel: string;
    placement: PluginViewPlacement;
    bounds?: PluginViewBounds | null;
    lifecycle: PluginViewLifecycle;
    permissions: string[];
};

/** WorkspaceHeader 和 PluginView 之间共享的状态 */
export type PluginViewState = {
    pluginId: string;
    commandId: string;
    pluginName: string;
    title: string;
    lifecycle: PluginViewLifecycle;
    placement: PluginViewPlacement;
    runtimeId: string | null;
};

export type ViewProvider = 'core' | { plugin: { pluginId: string } };
export type WindowHostKind = 'main' | 'detached';
export type SurfaceLifecycle = 'active' | 'hidden' | 'destroyed';

export type SurfaceMetadata = {
    id: string;
    webviewLabel: string;
    viewId: string;
    provider: ViewProvider;
    route: AppRoutePath;
    title: string;
    hostWindowLabel: string;
    hostKind: WindowHostKind;
    lifecycle: SurfaceLifecycle;
    focused: boolean;
    createdAt: string;
    updatedAt: string;
};

export type LauncherItem = {
    result: SearchResult;
    isPinned: boolean;
};

export type LauncherSection = {
    id: string;
    title: string;
    items: LauncherItem[];
};

export type LauncherPanelResponse = {
    sections: LauncherSection[];
};

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
    lastSummary?: ReloadIndexSummary | null;
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

export type AppSettings = {
    theme: ThemeValue;
    palette: {
        global_hotkey: string;
        show_recent: boolean;
        show_pinned: boolean;
    };
    search: {
        enabled_providers: string[];
    };
    window: {
        hide_on_blur: boolean;
        close_to_tray: boolean;
    };
};

export type ShortcutStatus = {
    accelerator: string;
    registered: boolean;
    error?: string | null;
};

export type UsageEvent = {
    target_type: string;
    target_id: string;
    query?: string | null;
    selected_at: string;
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
