// @litools/sdk —— 统一插件 SDK 入口

// ── 类型 ──
export type * from './types';

// ── 能力域 ──
export * as runtime from './runtime';
export * as storage from './storage';
export * as ui from './ui';
export * as commands from './commands';
export * as settings from './settings';
export * as diagnostics from './diagnostics';
export * as host from './host';
export * as search from './search';
export * as input from './input';

// ── 生命周期（preload 注入，不走 IPC） ──
export { onEnter, onLeave } from './runtime';
