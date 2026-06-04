# litools AI 协作说明

## 实现原则

实现每一个功能时，优先采用面向长期迭代的最佳方案。不要为了追求最小改动而引入 hack、飞线代码、临时兼容层或难以维护的局部补丁；当现有结构不适合承载新功能时，可以进行必要的重构，让实现保持清晰、可扩展、可维护。

## 前端组件库

项目桌面前端使用 SolidJS，并已引入 `@ark-ui/solid` 作为无样式组件库。

后续涉及 Ark UI 组件实现、用法查询或示例参考时，优先使用官方 LLMs 文档入口：

- https://ark-ui.com/llms.txt — 组件和文档链接结构化总览
- https://ark-ui.com/llms-full.txt — 完整文档、实现细节和示例
- https://ark-ui.com/llms-solid.txt — SolidJS 专用文档和实现细节

优先参考 `llms-solid.txt`，需要跨组件总览时再看 `llms.txt`，需要完整示例和细节时再看 `llms-full.txt`。

## 前端图标库

桌面前端已引入 `lucide-solid` 作为图标库。后续需要图标时，优先从 `lucide-solid` 导入对应图标组件，保持图标风格统一；不要临时内联 SVG 或混用其他图标库，除非已有图标库无法满足需求。

## 前端代码校验

每次更改桌面前端代码结束后，都需要运行前端代码格式化和校验：

- `pnpm --dir apps/desktop format`
- `pnpm --dir apps/desktop check`
