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
  icon?: string;
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
