export type SearchResultAction = {
    id: string;
    label: string;
};

export type SearchResult = {
    id: string;
    title: string;
    subtitle?: string | null;
    iconUri?: string | null;
    provider: string;
    score: number;
    actions: SearchResultAction[];
};

export type BuiltinCommandEffect = 'none' | 'openSettings' | 'reloadIndex' | 'openLogs' | 'quitApp' | 'toggleTheme';

export type CommandExecution = {
    resultId: string;
    actionId: string;
    message: string;
    effect: BuiltinCommandEffect;
};

export type AppSettings = {
    theme: 'system' | 'light' | 'dark' | string;
    palette: {
        global_hotkey: string;
        result_limit: number;
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
    recent_usage_count: number;
    recent_usage: UsageEvent[];
    settings: AppSettings;
    shortcut: ShortcutStatus;
};
