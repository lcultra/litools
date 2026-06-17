// ── 共享类型 —— @litools/sdk 单一真相来源 ──

// ---- runtime ----

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

// ---- storage ----

// ---- ui ----

// ---- commands ----

export type DynamicCommand = {
  id: string;
  title: string;
  subtitle?: string;
  keywords?: string[];
  mode?: 'instant' | 'view' | 'searchProvider';
  executor?: string;
  icon?: string;
  script?: string;
  persistence?: 'runtime' | 'session' | 'persistent'; // 默认 'session'
};

// ---- settings ----

export type ThemeValue = 'system' | 'light' | 'dark';

export type AppSettings = {
  theme: ThemeValue;
  palette: { global_hotkey: string; show_recent: boolean; show_pinned: boolean };
  search: { enabled_providers: string[] };
  window: { hide_on_blur: boolean; close_to_tray: boolean; center_on_show: boolean };
};

// ---- diagnostics ----

export type IndexStatus = {
  running: boolean;
  pending: boolean;
  lastTrigger?: string | null;
  lastError?: string | null;
  lastSummary?: unknown;
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

// ---- host ----

export type PluginCommandMode = 'instant' | 'view' | 'searchProvider';
export type PluginSource = 'bundled' | 'user';

export type PluginCommandSummary = {
  id: string; title: string; subtitle?: string | null; keywords: string[]; mode: PluginCommandMode;
};

export type PluginSummary = {
  id: string; name: string; version: string; description?: string | null; author?: string | null;
  icon: string; enabled: boolean; trusted: boolean; source: PluginSource; path: string;
  permissions: string[]; commands: PluginCommandSummary[];
};

export type PluginViewDescriptor = {
  pluginId: string; commandId: string; pluginName: string; title: string;
  entryUrl: string; icon: string; permissions: string[]; dev: boolean;
};

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

// ── InputContext (Phase 4A) ──

export type SearchFeature = {
  kind: string;
  source: string;
  confidence: number;
  metadata: Record<string, unknown>;
};

export type SearchAttachment = {
  kind: 'clipboard' | 'dragDrop' | 'filePath';
  data: number[];
  mimeType?: string;
  filename?: string;
};

export type InputContext = {
  version: number;
  raw: string;
  normalized: string;
  features: SearchFeature[];
  attachments: SearchAttachment[];
  metadata: Record<string, unknown>;
};

export type SearchRequest = {
  query: { text: string; limit?: number | null };
  context: InputContext;
  metadata: Record<string, unknown>;
};

export const FEATURE_KINDS = {
  JSON: 'json',
  URL: 'url',
  BASE64: 'base64',
  IMAGE: 'image',
  FILE: 'file',
  CURL: 'curl',
  JWT: 'jwt',
  UUID: 'uuid',
  COLOR: 'color',
  MARKDOWN: 'markdown',
} as const;

// ---- search / launcher types (host bridge also uses these) ----

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

export type CommandExecution = {
  resultId: string; actionId: string; message: string; effect: CommandEffect;
};

export type LauncherItem = { result: SearchResult; isPinned: boolean };
export type LauncherSection = { id: string; title: string; items: LauncherItem[] };
export type LauncherPanelResponse = { sections: LauncherSection[] };

export type ViewProvider = 'core' | { plugin: { pluginId: string } };
export type WindowHostKind = 'main' | 'detached';

export type BaseInfo = { mainWindowLabel: string; detachWindowPrefix: string };

export type SurfaceMetadata = {
  id: string; webviewLabel: string; viewId: string; provider: ViewProvider; route: string;
  title: string; hostWindowLabel: string; hostKind: WindowHostKind;
  lifecycle: 'active' | 'hidden' | 'destroyed'; focused: boolean; createdAt: string; updatedAt: string;
};
