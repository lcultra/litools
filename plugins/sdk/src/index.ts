export type PluginPermission =
  | 'clipboard:read'
  | 'clipboard:write'
  | 'files:open'
  | 'files:reveal'
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
};

declare global {
  interface Window {
    litools: LitoolsRuntimeApi;
  }
}
