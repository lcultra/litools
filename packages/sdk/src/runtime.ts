import { invoke } from '@tauri-apps/api/core';
import type { PluginRuntimeInfo } from './types';

const SDK = 'plugin:litools-sdk';

// ---- IPC helpers ----

function invokeSdk<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(`${SDK}|${cmd}`, args);
}

// ---- runtime API ----

export function ready(): Promise<PluginRuntimeInfo> {
  return invokeSdk('sdk_runtime_ready');
}

export function getInfo(): Promise<PluginRuntimeInfo> {
  return invokeSdk('sdk_runtime_get_info');
}

export function queryPermission(permission: string): Promise<{ permission: string; state: 'granted' | 'denied' }> {
  return invokeSdk('sdk_permissions_query', { permission });
}

// ---- lifecycle (preload 注入，不走 IPC) ----

export function onEnter(cb: () => void): () => void {
  const listeners = (window as any).__litoolsLifecycleListeners;
  if (!listeners) throw new Error('litools runtime not available');
  listeners.enter.add(cb);
  return () => listeners.enter.delete(cb);
}

export function onLeave(cb: () => void): () => void {
  const listeners = (window as any).__litoolsLifecycleListeners;
  if (!listeners) throw new Error('litools runtime not available');
  listeners.leave.add(cb);
  return () => listeners.leave.delete(cb);
}

// re-export invoke helper for other SDK modules
export { invokeSdk };
