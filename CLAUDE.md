# litools AI 协作说明

litools 是基于 Rust / Tauri / SolidJS 的桌面效率工具平台，目标是做成 uTools 风格的本地启动器，并逐步扩展为插件驱动的本地工具生态。Roadmap 见 [README.md](README.md)。

任何编码 Agent 开始工作前都应先读完本文件，并在实现中遵循这里的工程结构和跨层约定。

## 核心原则

实现每一个功能时，优先采用面向长期迭代的最佳方案。

- 不要为了追求最小改动而引入 hack、飞线代码、临时兼容层或难以维护的局部补丁。
- 当现有结构不适合承载新功能时，进行必要的重构，让实现保持清晰、可扩展、可维护。
- 在正式版发布之前，所有改动都应采用长期方案，不要害怕重构。现阶段没有历史用户和旧配置需要兼容，不要为不存在的旧安装做兼容层或迁移逻辑。
- 新增能力时先判断它属于哪一层（见下方分层），把代码放进正确的 crate / 模块，而不是就近塞进调用点。

## 工程结构

这是一个 Cargo workspace + 单个 pnpm 桌面前端的混合仓库。

```
crates/                  Rust 业务 crate（与 UI 无关，可独立测试）
  litools-config         共享常量与配置：窗口标签、view ID、事件名、协议 scheme、默认值等
  litools-core           应用编排核心：LitoolsApp / AppContext，聚合所有下层 crate
  litools-index          SQLite 存储：IndexDatabase、repository、schema、migrations
  litools-search         搜索引擎：engine / matcher / provider / ranking / query
  litools-plugin         插件体系：manifest / permission / discovery / manager / ids
  litools-settings       设置存储：settings / storage
  litools-system         系统适配：app/file 索引、剪贴板、热键、拼音、platform/{macos,linux,windows}
  litools-telemetry      可观测性：logging
apps/desktop             SolidJS 前端
  src/                   前端代码（详见下方前端约定）
  src-tauri/             Tauri 桌面壳：IPC、窗口、surface、plugin_runtime、tray、自定义协议
plugins/                 插件生态
  sdk/                   插件开发 SDK（TypeScript）
  bundled/               随应用打包的内置插件
migrations/              数据库迁移 SQL
tests/                   测试
  integration/           集成测试
  fixtures/              测试 fixture
  e2e/                   端到端测试
schemas/                 JSON Schema 定义
  settings.schema.json
  marketplace.schema.json
  plugin-manifest.schema.json
docs/                    架构与子系统文档
```

### 后端分层与依赖方向

依赖只能从上往下，下层 crate 不得反向依赖上层：

```
apps/desktop/src-tauri  →  litools-core  →  litools-{index,search,plugin,settings,system,telemetry}
                        →  litools-config   (leaf crate，无 litools 内部依赖)
```

- `litools-config` 是最底层的共享常量 crate，定义窗口/ webview 标签前缀、view ID、事件名称、协议 scheme、窗口默认尺寸等。所有需要共享字符串常量的 crate 都可以依赖它。
- 业务编排逻辑放在 `litools-core`（`LitoolsApp` 是唯一的应用入口对象，聚合 database / search / plugins / settings）。
- 与 Tauri、窗口、IPC 相关的代码只允许出现在 `apps/desktop/src-tauri`，不要让 Tauri 类型泄漏进 `crates/`。
- 跨 crate 共享的依赖版本统一在根 `Cargo.toml` 的 `[workspace.dependencies]` 声明，子 crate 用 `workspace = true` 引用，不要在子 crate 里写死版本。
- Rust edition 为 `2024`。

### desktop 后端模块组织

`src-tauri/src` 下的子系统遵循统一的分层模式，新增子系统时沿用：

| 模块 | 职责 |
|------|------|
| `state.rs` | `AppState` — 所有可变运行时状态的唯一持有者，内部用 `Mutex` 包裹各注册表 |
| `main.rs` | Tauri 入口：协议注册、插件注册、setup、IPC handler 注册、事件处理 |
| `ipc/` | `#[tauri::command]` 入口（按领域拆分：`launcher.rs`、`surface.rs`、`diagnostics.rs`、`settings.rs`、`plugins.rs`） |
| `surface/` | Surface 子系统：窗口内的 webview 生命周期（model / registry / service / events） |
| `plugin_runtime/` | 插件运行时子系统：插件 webview 的创建/停靠/分离/销毁（model / registry / service / ipc / permissions / preload） |
| `view/` | 视图定义与路由校验：ViewDefinition、route → view 映射、插件路由解析（model / registry） |
| `windowing/` | 原生窗口操作：窗口/webview 创建、标签生成、生命周期事件分发、定位、webview reparent（labels / lifecycle / native / positioning / reparent） |
| `protocol/` | 自定义 URI scheme 协议：icon 协议、插件资源协议、图标缓存（icon / icon_cache / plugin） |
| `shortcut.rs` | 全局快捷键注册与匹配 |
| `tray.rs` | 系统托盘 |
| `app_watcher.rs` | 应用目录文件变更监听（/Applications 等） |
| `index_refresh.rs` | 索引刷新触发与状态管理 |
| `macos_icon.rs` | macOS 平台图标提取（条件编译） |

各子系统的内部文件约定：

- `model.rs` — 数据结构与生命周期状态枚举
- `registry.rs` — 线程内状态注册表（由 `AppState` 持有并加锁）
- `service.rs` — 业务行为（窗口创建、reparent、状态迁移等）
- `ipc.rs` — `#[tauri::command]` 入口，只做参数解析和 service 调用
- 需要时再加 `events.rs`（前端事件广播）、`permissions.rs`、`preload.rs`

`AppState`（[state.rs](apps/desktop/src-tauri/src/state.rs)）持有以下可变状态：

- `app: Mutex<LitoolsApp>` — 核心应用实例
- `surfaces: Mutex<SurfaceRegistry>` — 所有 surface 注册信息
- `plugin_runtimes: Mutex<PluginRuntimeRegistry>` — 所有插件运行时注册信息
- `pooled_detached: Mutex<Option<String>>` — 预创建的分离窗口标签池，加速插件 detach
- `index_status: Mutex<IndexStatus>` — 索引刷新状态
- `shortcut_status: Mutex<ShortcutStatus>` — 快捷键注册状态
- `app_watcher: AppWatcherState` + `app_watcher_handle` — 文件监听状态

不要在 `AppState` 之外另起全局可变状态。

### 窗口 / Webview 标签体系

所有窗口标签和 webview 标签的统一前缀定义在 `litools-config` 的 [labels.rs](crates/litools-config/src/labels.rs)，由 `windowing::labels` 模块 re-export 并提供辅助函数：

| 常量 | 值 | 用途 |
|------|-----|------|
| `MAIN_WINDOW_LABEL` | `"main"` | 主窗口，整个应用只有一个 |
| `DETACHED_PANEL_WINDOW_PREFIX` | `"detached-panel-"` | 分离面板窗口标签前缀 |
| `PLUGIN_WINDOW_PREFIX` | `"plugin-window-"` | 插件分离窗口标签前缀 |
| `PLUGIN_WEBVIEW_PREFIX` | `"plugin-"` | 插件内容 webview 标签前缀 |
| `SURFACE_WEBVIEW_LABEL_PREFIX` | `"surface-"` | Surface webview 标签前缀 |
| `CORE_LAUNCHER_VIEW_ID` | `"core.launcher"` | 启动器视图 ID |

`windowing::labels` 同时提供 `is_detached_panel_window_label`、`is_plugin_window_label` 等判断函数，以及 `surface_webview_label(id)`、`plugin_window_label(id)` 等标签生成函数。

### 新增 IPC 命令的完整链路

一个 IPC 命令必须四处保持一致，缺一不可：

1. 在 `ipc/*.rs`（通用 IPC）或子系统的 `ipc.rs`（如 `plugin_runtime::ipc`）写 `#[tauri::command]` 函数。
2. 在 [main.rs](apps/desktop/src-tauri/src/main.rs) 的 `tauri::generate_handler!` 列表中注册。
3. 在 [bridge/commands.ts](apps/desktop/src/bridge/commands.ts) 加一个类型化 wrapper（前端只通过 bridge 调用 `invoke`，组件里不直接写 `invoke`）。
4. 在 [bridge/types.ts](apps/desktop/src/bridge/types.ts) 补齐请求 / 响应类型。

### 后端数据库锁

`litools-index` 的 `IndexDatabase` 使用 `Arc<Mutex<rusqlite::Connection>>` 管理数据库连接，标准库 `Mutex` 不可重入。后端实现中不要在持有 `self.context.database.connection()` 返回的 guard 时，再调用任何可能间接获取数据库连接的 helper（例如 result 校验、launcher item 构建、repository 查询封装等），否则会在同一线程再次 lock 数据库 mutex 并导致界面卡死。

涉及数据库写入且需要先做业务校验时，优先分阶段处理：先在不持有数据库连接 guard 的情况下完成 result id 解析、命令校验等逻辑；再获取数据库连接，集中执行 repository 读取 / 写入和事务。典型案例是已固定排序持久化：不要在持有 `PinnedRepository` / connection 的同时调用 `validated_target_from_result_id`，因为 app result 校验会再次读取数据库。

## 前端约定

桌面前端使用 SolidJS + `@solidjs/router`，UI 用 `@ark-ui/solid`（无样式组件库）+ Tailwind CSS v4，图标用 `lucide-solid`。

### 代码组织

```
apps/desktop/src/
  main.tsx               入口：render App
  App.tsx                根组件：HashRouter + 路由定义 + 主题/全局事件
  styles.css             全局样式
  bridge/                所有 Tauri IPC 桥接层
    commands.ts           类型化 invoke wrapper
    events.ts             类型化事件监听
    types.ts              请求/响应类型定义
  shared/                跨 feature 复用的公共模块
    routes.ts             路由常量和生成函数（PLUGIN_ROUTE_PATTERN、generatePluginPath）
    store.ts              模块级 signals（hostWindowLabel、settings），通过 createRoot 持久化
    theme.ts              主题工具
    strings.ts            字符串工具
  hooks/                 全局 hooks
    useAppEvents.ts       Tauri 事件 → shared store 的桥接
  components/            通用 UI 组件
    Button.tsx
    PageState.tsx
    WindowFrame.tsx       窗口边框（拖拽区域）
    WorkspaceHeader.tsx   插件视图标题栏
  features/              页面 / 功能目录
    launcher/             启动器
      LauncherPage.tsx      路由页面壳（加载设置、协调子组件）
      LauncherView.tsx      搜索+结果展示主视图
      LauncherInput.tsx     搜索输入框
      PinnedSortableGrid.tsx 已固定项的排序网格
      HighlightedText.tsx   匹配关键词高亮
      useLauncherNavigation.ts 键盘导航 hook
    workspace/            插件工作区
      WorkspacePage.tsx     插件命令视图承载页
```

- 跨 feature 复用的公共 helper 放在 `shared/`；单 feature 私有 helper 放在对应 feature 目录；组件私有逻辑保留在组件文件内。
- 通用 UI 组件放在 `components/`，不要创建无边界的 `helpers` / `utils` 大杂烩。
- 每个页面 / 功能放在 `features/` 下的独立目录。
- 所有 Tauri IPC 调用统一经过 `bridge/`（`commands` / `events` / `types`），组件不直接 `invoke`。

### 路由

使用 `HashRouter`，路由定义在 [App.tsx](apps/desktop/src/App.tsx)：

| 路由 | 组件 | 说明 |
|------|------|------|
| `/` | `LauncherPage` | 启动器主页 |
| `/plugin/:pluginId/:commandId` | `WorkspacePage` | 插件命令视图 |

路由常量和生成函数集中在 [shared/routes.ts](apps/desktop/src/shared/routes.ts)：
- `PLUGIN_ROUTE_PATTERN` — 插件路由模式字符串
- `generatePluginPath(pluginId, commandId)` — 生成插件路由路径

后端 `view/registry.rs` 负责对应的 route 合法性校验，两边的路由集合需保持一致。`litools_config::labels::CORE_LAUNCHER_VIEW_ID` 是前后端统一的启动器视图 ID。

### 状态管理

- 默认不引入全局状态库；页面状态优先用本地 `createSignal` / `createResource`。
- 需要跨组件/跨 feature 一致读写的 App 级状态（当前仅 `hostWindowLabel` 和 `settings`）放在 `shared/store.ts`，通过 `createRoot` 包裹的模块级 signal 实现，不依赖组件树生命周期。
- 不要把页面编辑态、搜索态、诊断 resource、表单草稿、loading / error 等临时状态提升为全局状态。

### 图标

需要图标时优先从 `lucide-solid` 导入，保持风格统一；不要临时内联 SVG 或混用其他图标库，除非已有图标库无法满足需求。

### 代码校验

每次更改桌面前端代码结束后，都要运行格式化和校验（基于 Biome）：

- `pnpm --dir apps/desktop format`
- `pnpm --dir apps/desktop check`
- `pnpm --dir apps/desktop exec tsc --noEmit`

## 文档参考入口

涉及 Ark UI 组件实现、用法查询或示例参考时，优先使用官方 LLMs 文档：

- https://ark-ui.com/llms-solid.txt — SolidJS 专用文档和实现细节（首选）
- https://ark-ui.com/llms.txt — 组件和文档链接结构化总览（需要跨组件总览时）
- https://ark-ui.com/llms-full.txt — 完整文档、实现细节和示例（需要完整示例时）

涉及 SolidJS API、响应式模型、组件模式或最佳实践时，优先参考：

- https://docs.solidjs.com/llms.txt — SolidJS 官方文档结构化入口
