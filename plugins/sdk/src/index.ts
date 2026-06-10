export type PluginPermission =
  | 'clipboard:read'
  | 'clipboard:write'
  | 'diagnostics:read'
  | 'files:open'
  | 'files:reveal'
  | 'plugins:list'
  | 'settings:read'
  | 'settings:write'
  | 'storage:plugin'
  | 'ui:toast'
  | 'ui:window';

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
  development?: {
    /** dev server 入口 URL，如 "http://127.0.0.1:5173/index.html" */
    main: string;
  };
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

export type PluginRuntimeBounds = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type PluginRuntimeInfo = {
  runtimeId: string;
  pluginId: string;
  commandId: string;
  pluginName: string;
  title: string;
  hostWindowLabel: string;
  detachedWindowLabel?: string | null;
  headerWebviewLabel?: string | null;
  webviewLabel: string;
  placement: PluginRuntimePlacement;
  bounds?: PluginRuntimeBounds | null;
  lifecycle: PluginRuntimeLifecycle;
  permissions: string[];
};

export type PermissionQueryResult = {
  permission: string;
  state: 'granted' | 'denied';
};

export type LitoolsRuntimeApi = {
  runtime: {
    ready(): Promise<PluginRuntimeInfo>;
    getInfo(): Promise<PluginRuntimeInfo>;
  };
  permissions: {
    query(permission: PluginPermission | string): Promise<PermissionQueryResult>;
  };
  lifecycle: {
    onEnter(callback: () => void): () => void;
    onLeave(callback: () => void): () => void;
  };
  ui: {
    close(): Promise<void>;
    setTitle(title: string): Promise<void>;
    toast(message: string, options?: Record<string, unknown>): Promise<void>;
  };
  storage: {
    get<T = unknown>(key: string): Promise<T | null>;
    set(key: string, value: unknown): Promise<void>;
    remove(key: string): Promise<void>;
    clear(): Promise<void>;
  };
  settings: {
    get(): Promise<AppSettings>;
    update(settings: AppSettings): Promise<AppSettings>;
  };
  diagnostics: {
    get(): Promise<DiagnosticsResponse>;
  };
  plugins: {
    list(): Promise<PluginSummary[]>;
  };
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
    id: string;
    title: string;
    subtitle?: string | null;
    keywords: string[];
    mode: string;
  }>;
};

export type AppSettings = {
  theme: 'system' | 'light' | 'dark';
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
    center_on_show: boolean;
  };
};

export type DiagnosticsResponse = {
  app_version: string;
  app_data_dir: string;
  platform: string;
  plugin_count: number;
  command_count: number;
  app_count: number;
  index_status: {
    running: boolean;
    pending: boolean;
    lastTrigger?: string | null;
    lastError?: string | null;
    lastSummary?: Record<string, unknown> | null;
  };
  last_persisted_index_status?: Record<string, unknown> | null;
  app_watcher: {
    platform: string;
    enabled: boolean;
    status: string;
    watchedPaths: string[];
    error?: string | null;
  };
  icon_cache: {
    fileCount: number;
    totalBytes: number;
    maxFiles: number;
    maxBytes: number;
    lastPrunedAt?: string | null;
    lastPrunedFiles: number;
    error?: string | null;
  };
  recent_usage_count: number;
  recent_usage: Array<{
    target_type: string;
    target_id: string;
    query?: string | null;
    selected_at: string;
  }>;
  settings: AppSettings;
  shortcut: {
    accelerator: string;
    registered: boolean;
    error?: string | null;
  };
};

declare global {
  interface Window {
    litools: LitoolsRuntimeApi;
  }
}
