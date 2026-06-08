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
    mode TEXT NOT NULL,
    permission_requirements TEXT NOT NULL DEFAULT '[]',
    FOREIGN KEY(plugin_id) REFERENCES plugins(id)
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

CREATE TABLE IF NOT EXISTS permission_grants (
    plugin_id TEXT NOT NULL,
    permission TEXT NOT NULL,
    granted INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY(plugin_id, permission)
);
"#;
