import { invokeSdk } from './runtime';

// ---- storage API ----

export function get<T = unknown>(key: string): Promise<T | null> {
  return invokeSdk<T>('sdk_storage_get', { key });
}

export function set(key: string, value: unknown): Promise<void> {
  return invokeSdk('sdk_storage_set', { key, value });
}

export function remove(key: string): Promise<void> {
  return invokeSdk('sdk_storage_remove', { key });
}

export function clear(): Promise<void> {
  return invokeSdk('sdk_storage_clear');
}
