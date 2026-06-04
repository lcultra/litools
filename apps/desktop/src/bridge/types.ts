export type SearchResultAction = {
  id: string;
  label: string;
};

export type SearchResult = {
  id: string;
  title: string;
  subtitle?: string | null;
  provider: string;
  score: number;
  actions: SearchResultAction[];
};

export type BuiltinCommandEffect =
  | 'none'
  | 'openSettings'
  | 'reloadIndex'
  | 'openLogs'
  | 'quitApp'
  | 'toggleTheme';

export type CommandExecution = {
  resultId: string;
  actionId: string;
  message: string;
  effect: BuiltinCommandEffect;
};

export type AppSettings = {
  theme: string;
  palette: {
    global_hotkey: string;
    result_limit: number;
  };
  search: {
    enabled_providers: string[];
  };
};

export type UsageEvent = {
  target_type: string;
  target_id: string;
  query?: string | null;
  selected_at: string;
};

export type DiagnosticsResponse = {
  app_version: string;
  plugin_count: number;
  recent_usage: UsageEvent[];
};
