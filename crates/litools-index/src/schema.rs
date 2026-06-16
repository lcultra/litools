pub const INITIAL_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS apps (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    icon_path TEXT,
    localized_names_json TEXT NOT NULL DEFAULT '[]',
    aliases_json TEXT NOT NULL DEFAULT '[]',
    search_text TEXT NOT NULL DEFAULT '',
    platform TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    launch_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS commands (
    id TEXT PRIMARY KEY,
    namespace TEXT NOT NULL,
    title TEXT NOT NULL,
    subtitle TEXT,
    action TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS plugins (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    path TEXT NOT NULL,
    manifest_json TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'user',
    enabled INTEGER NOT NULL DEFAULT 1,
    trusted INTEGER NOT NULL DEFAULT 0,
    installed_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS plugin_commands (
    id TEXT PRIMARY KEY,
    plugin_id TEXT NOT NULL,
    command_id TEXT NOT NULL,
    title TEXT NOT NULL,
    subtitle TEXT,
    keywords TEXT NOT NULL DEFAULT '[]',
    mode TEXT NOT NULL DEFAULT 'instant',
    executor TEXT,
    icon TEXT,
    script TEXT,
    source TEXT NOT NULL DEFAULT 'manifest',
    lifecycle TEXT NOT NULL DEFAULT 'permanent',
    registrar_runtime_id TEXT,
    executor_runtime_id TEXT,
    permission_requirements TEXT NOT NULL DEFAULT '[]',
    FOREIGN KEY(plugin_id) REFERENCES plugins(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS usage_events (
    id TEXT PRIMARY KEY,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    query TEXT,
    selected_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS pinned_items (
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    pinned_at TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (target_type, target_id)
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS index_metadata (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS plugin_storage (
    plugin_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY(plugin_id, key),
    FOREIGN KEY(plugin_id) REFERENCES plugins(id) ON DELETE CASCADE
);

-- 插件命令查询索引
CREATE INDEX IF NOT EXISTS idx_plugin_commands_plugin ON plugin_commands(plugin_id);
CREATE INDEX IF NOT EXISTS idx_plugin_commands_lifecycle ON plugin_commands(lifecycle);
CREATE INDEX IF NOT EXISTS idx_plugin_commands_runtime ON plugin_commands(registrar_runtime_id);

-- 使用事件排序/去重索引（搜索核心路径）
CREATE INDEX IF NOT EXISTS idx_usage_events_selected ON usage_events(selected_at);
CREATE INDEX IF NOT EXISTS idx_usage_events_target ON usage_events(target_type, target_id, selected_at);
"#;
