import { invokeSdk } from './runtime';

// ---- ui API ----

export function close(): Promise<void> {
  return invokeSdk('sdk_ui_close');
}

export function setTitle(title: string): Promise<void> {
  return invokeSdk('sdk_ui_set_title', { title });
}

export function toast(message: string, options?: Record<string, unknown>): Promise<void> {
  return invokeSdk('sdk_ui_toast', { message, options });
}
