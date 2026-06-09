//! 后端静态中文文案。
//!
//! 所有用户可读的字符串集中管理，未来可用于 i18n。

pub mod commands {
    // ── 内置命令标题 & 副标题 ──

    /// 标题：重载索引
    pub const RELOAD_INDEX_TITLE: &str = "重载索引";
    /// 副标题：刷新本地搜索索引
    pub const RELOAD_INDEX_SUBTITLE: &str = "刷新本地搜索索引";
    /// 标题：打开日志目录
    pub const OPEN_LOGS_TITLE: &str = "打开日志目录";
    /// 副标题：在系统文件管理器中打开日志目录
    pub const OPEN_LOGS_SUBTITLE: &str = "在系统文件管理器中打开日志目录";
    /// 标题：打开数据目录
    pub const OPEN_DATA_TITLE: &str = "打开数据目录";
    /// 副标题：在系统文件管理器中打开本地数据目录
    pub const OPEN_DATA_SUBTITLE: &str = "在系统文件管理器中打开本地数据目录";
    /// 标题：退出应用
    pub const QUIT_APP_TITLE: &str = "退出应用";
    /// 副标题：退出 litools
    pub const QUIT_APP_SUBTITLE: &str = "退出 litools";
    /// 标题：切换主题
    pub const TOGGLE_THEME_TITLE: &str = "切换主题";
    /// 副标题：在浅色和深色主题之间切换
    pub const TOGGLE_THEME_SUBTITLE: &str = "在浅色和深色主题之间切换";
}

pub mod launcher {
    // ── 启动器面板分类标题 ──

    /// 有搜索关键词时显示的分类名
    pub const SECTION_BEST: &str = "最佳搜索结果";
    /// 空搜索时 "最近使用" 分类名
    pub const SECTION_RECENT: &str = "最近使用";
    /// 空搜索时 "已固定" 分类名
    pub const SECTION_PINNED: &str = "已固定";
}

pub mod effects {
    // ── 命令执行结果消息 ──

    /// 未执行任何操作
    pub const NONE: &str = "未执行任何操作";
    /// 正在打开日志目录
    pub const OPENING_LOGS: &str = "正在打开日志目录";
    /// 正在打开数据目录
    pub const OPENING_DATA: &str = "正在打开数据目录";
    /// 正在打开插件
    pub const OPENING_PLUGIN: &str = "正在打开插件";
    /// 正在重载索引
    pub const RELOADING_INDEX: &str = "正在重载索引";
    /// 正在退出应用
    pub const QUITTING: &str = "正在退出应用";
    /// 正在切换主题
    pub const TOGGLING_THEME: &str = "正在切换主题";
}

pub mod actions {
    // ── 操作按钮标签 ──

    /// 打开（应用/插件视图）
    pub const OPEN: &str = "打开";
    /// 执行（内置命令）
    pub const EXECUTE: &str = "执行";
}
