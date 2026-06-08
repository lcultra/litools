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
  litools-core           应用编排核心：LitoolsApp / AppContext，聚合所有下层 crate
  litools-index          SQLite 存储：IndexDatabase、repository、schema、migrations
  litools-search         搜索引擎：engine / provider / matcher / ranking / query
  litools-plugin         插件体系：manifest / permission / discovery / manager / runtime / marketplace
  litools-settings       设置存储：settings / profile / storage
  litools-system         系统适配：app/file 索引、剪贴板、热键、platform/{macos,linux,windows}
  litools-telemetry      可观测性：logging / diagnostics / metrics
apps/desktop             SolidJS 前端
  src/                   前端代码（详见下方前端约定）
  src-tauri/             Tauri 桌面壳：IPC、窗口、surface、plugin_runtime、tray、自定义协议
plugins/                 插件生态
  sdk/                   插件开发 SDK（TypeScript）
  bundled/               随应用打包的内置插件
migrations/              数据库迁移
docs/                    架构与子系统文档
```

### 后端分层与依赖方向

依赖只能从上往下，下层 crate 不得反向依赖上层：

```
apps/desktop/src-tauri  →  litools-core  →  litools-{index,search,plugin,settings,system,telemetry}
```

- 业务编排逻辑放在 `litools-core`（`LitoolsApp` 是唯一的应用入口对象，`AppContext` 持有 database / search / plugins / settings）。
- 与 Tauri、窗口、IPC 相关的代码只允许出现在 `apps/desktop/src-tauri`，不要让 Tauri 类型泄漏进 `crates/`。
- 跨 crate 共享的依赖版本统一在根 `Cargo.toml` 的 `[workspace.dependencies]` 声明，子 crate 用 `workspace = true` 引用，不要在子 crate 里写死版本。

### desktop 后端模块组织

`src-tauri/src` 下的子系统（如 `surface`、`plugin_runtime`、`view`）遵循统一的分层模式，新增子系统时沿用：

- `model.rs` — 数据结构与生命周期状态枚举
- `registry.rs` — 线程内状态注册表（由 `AppState` 持有并加锁）
- `service.rs` — 业务行为（窗口创建、reparent、状态迁移等）
- `ipc.rs` — `#[tauri::command]` 入口，只做参数解析和 service 调用
- 需要时再加 `events.rs`（前端事件广播）、`permissions.rs`、`preload.rs`

`AppState`（[state.rs](apps/desktop/src-tauri/src/state.rs)）是所有可变运行时状态的唯一持有者，内部用 `Mutex` 包裹各注册表；不要在 `AppState` 之外另起全局可变状态。

### 新增 IPC 命令的完整链路

一个 IPC 命令必须四处保持一致，缺一不可：

1. 在对应的 `ipc/*.rs`（或子系统的 `ipc.rs`）写 `#[tauri::command]` 函数。
2. 在 [main.rs](apps/desktop/src-tauri/src/main.rs) 的 `tauri::generate_handler!` 列表中注册。
3. 在 [bridge/commands.ts](apps/desktop/src/bridge/commands.ts) 加一个类型化 wrapper（前端只通过 bridge 调用 `invoke`，组件里不直接写 `invoke`）。
4. 在 [bridge/types.ts](apps/desktop/src/bridge/types.ts) 补齐请求 / 响应类型。

### 后端数据库锁

`litools-index` 的 `IndexDatabase` 使用 `Arc<Mutex<rusqlite::Connection>>` 管理数据库连接，标准库 `Mutex` 不可重入。后端实现中不要在持有 `self.context.database.connection()` 返回的 guard 时，再调用任何可能间接获取数据库连接的 helper（例如 result 校验、launcher item 构建、repository 查询封装等），否则会在同一线程再次 lock 数据库 mutex 并导致界面卡死。

涉及数据库写入且需要先做业务校验时，优先分阶段处理：先在不持有数据库连接 guard 的情况下完成 result id 解析、命令校验等逻辑；再获取数据库连接，集中执行 repository 读取 / 写入和事务。典型案例是已固定排序持久化：不要在持有 `PinnedRepository` / connection 的同时调用 `validated_target_from_result_id`，因为 app result 校验会再次读取数据库。

## 前端约定

桌面前端使用 SolidJS + `@solidjs/router`，UI 用 `@ark-ui/solid`（无样式组件库）+ Tailwind CSS v4，图标用 `lucide-solid`。

### 代码组织

- 跨 feature 复用的公共 helper 放在 [src/shared/](apps/desktop/src/shared/)；单 feature 私有 helper 放在对应 feature 目录；组件私有逻辑保留在组件文件内。
- 通用 UI 组件放在 [src/components/](apps/desktop/src/components/)，不要创建无边界的 `helpers` / `utils` 大杂烩。
- 每个页面 / 功能放在 [src/features/](apps/desktop/src/features/) 下的独立目录。
- 所有 Tauri IPC 调用统一经过 [src/bridge/](apps/desktop/src/bridge/)（`commands` / `events` / `types`），组件不直接 `invoke`。

### 路由与窗口视图的单一事实源

[src/views/registry.ts](apps/desktop/src/views/registry.ts) 是前端路由、视图类型（launcher / panel / runtime）和窗口宿主（main / detached / runtime）的单一事实源。新增页面或调整某页面能否独立成窗时，改这里的 `routeDefinitions`，不要在组件里散落硬编码路径。后端 `view/registry.rs` 负责对应的 route 合法性校验，两边的路由集合需保持一致。

### 状态管理

- 默认不引入全局状态库；页面状态优先用本地 `createSignal` / `createResource`。
- 只有当状态需要跨 feature 一致读写或避免多层 props 透传时，才用 Solid Context 承载 App 级 signals / stores / actions。
- 不要把页面编辑态、搜索态、诊断 resource、表单草稿、loading / error 等临时状态提升为全局状态。

### 图标

需要图标时优先从 `lucide-solid` 导入，保持风格统一；不要临时内联 SVG 或混用其他图标库，除非已有图标库无法满足需求。

### 代码校验

每次更改桌面前端代码结束后，都要运行格式化和校验（基于 Biome）：

- `pnpm --dir apps/desktop format`
- `pnpm --dir apps/desktop check`

## 文档参考入口

涉及 Ark UI 组件实现、用法查询或示例参考时，优先使用官方 LLMs 文档：

- https://ark-ui.com/llms-solid.txt — SolidJS 专用文档和实现细节（首选）
- https://ark-ui.com/llms.txt — 组件和文档链接结构化总览（需要跨组件总览时）
- https://ark-ui.com/llms-full.txt — 完整文档、实现细节和示例（需要完整示例时）

涉及 SolidJS API、响应式模型、组件模式或最佳实践时，优先参考：

- https://docs.solidjs.com/llms.txt — SolidJS 官方文档结构化入口
