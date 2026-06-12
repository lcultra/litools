# litools AI 协作说明

litools 是基于 Rust / Tauri / SolidJS 的桌面效率工具平台，目标是做成 uTools 风格的本地启动器，并逐步扩展为插件驱动的本地工具生态。Roadmap 见 [README.md](README.md)。

任何编码 Agent 开始工作前都应先读完本文件，并在实现中遵循这里的工程结构和跨层约定。

## 核心原则

实现每一个功能时，优先采用面向长期迭代的最佳方案。

- 不要为了追求最小改动而引入 hack、飞线代码、临时兼容层或难以维护的局部补丁。
- 当现有结构不适合承载新功能时，进行必要的重构，让实现保持清晰、可扩展、可维护。
- 在正式版发布之前，所有改动都应采用长期方案，不要害怕重构。现阶段没有历史用户和旧配置需要兼容，不要为不存在的旧安装做兼容层或迁移逻辑。
- 新增能力时先判断它属于哪一层（见下方分层），把代码放进正确的 crate / 模块，而不是就近塞进调用点。
- 规划结果和代码注释用中文，思考过程无所谓。
- **不要自动 `git commit`**。写完代码后等用户确认再提交，禁止在无人干预的情况下自行提交。

## 工程结构

```
crates/                    Rust 业务 crate（与 UI 无关）
  litools-config           共享常量（leaf crate，无内部依赖）
  litools-core             应用编排（LitoolsApp），聚合下层 crate
  litools-{index,search,plugin,settings,system,telemetry}
apps/desktop/src-tauri     Tauri 桌面壳
apps/desktop/src/          SolidJS 前端
plugins/                   插件生态（sdk/ + bundled/）
```

**依赖方向（只能从上往下）：**

```
apps/desktop/src-tauri  →  litools-core  →  litools-{index,search,plugin,settings,system,telemetry}
                        →  litools-config
```

**约束：**

- Tauri 类型只在 `apps/desktop/src-tauri`，不得泄漏进 `crates/`。
- 跨 crate 依赖版本在根 `Cargo.toml` `[workspace.dependencies]` 声明，子 crate 用 `workspace = true`。
- Rust edition `2024`。
- `AppState` 是所有可变状态的唯一持有者，不要另起全局可变状态。
- 子系统文件按约定命名：`model.rs` / `registry.rs` / `service.rs` / `ipc.rs`，需要时加 `events.rs` / `permissions.rs` / `preload.rs`。
- 窗口/webview 标签前缀定义在 `litools-config` [labels.rs](crates/litools-config/src/labels.rs)，不得硬编码字符串。
- 新增 IPC：`#[tauri::command]` → `main.rs` `generate_handler!` → `bridge/commands.ts` wrapper → `bridge/types.ts` 类型，缺一不可。
- 数据库 `Arc<Mutex<Connection>>` 不可重入：持锁期间不得调用可能再次获取连接的 helper，否则卡死。写入前先不持锁校验，再持锁读写。

## 前端约定

SolidJS + `@solidjs/router` + `@ark-ui/solid` + Tailwind CSS v4 + `lucide-solid`。

**约束：**

- 路由用 `HashRouter`，定义在 [App.tsx](apps/desktop/src/App.tsx)：`/` → `LauncherPage`，`/plugin/:pluginId/:commandId` → `WorkspacePage`。路由常量集中在 [shared/routes.ts](apps/desktop/src/shared/routes.ts)，后端 `view/registry.rs` 做对应校验，两边路由集合必须一致。
- 状态：默认不引入全局状态库，页面用本地 `createSignal` / `createResource`。仅 `hostWindowLabel`、`settings` 等 App 级状态放 `shared/store.ts`（`createRoot` 模块级 signal）。不要把页面临时状态提升为全局。
- IPC 调用统一走 `bridge/`（`commands.ts` / `types.ts`），组件不直接 `invoke`。
- 图标优先 `lucide-solid`，不混用其他图标库。
- 每次改前端后运行：`pnpm --dir apps/desktop format` → `pnpm --dir apps/desktop check` → `pnpm --dir apps/desktop exec tsc --noEmit`。

## 文档参考入口

涉及 Tauri API、窗口管理、IPC、事件系统等后端问题需要查文档时，优先从本地 rustdoc 查找：

- `.claude/docs/rustdoc-tauri-2.11.2/tauri/` — Tauri 2.11.2 完整 API 文档（`all.html` 为入口，支持按 crate/module 浏览）
- 入口文件 `help.html` 列出所有 crate 文档索引


涉及 SolidJS API、响应式模型、组件模式或最佳实践时，优先参考：

- https://docs.solidjs.com/llms.txt — SolidJS 官方文档结构化入口
