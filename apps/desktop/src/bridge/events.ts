import { listen } from '@tauri-apps/api/event';

export function onFocusSearch(handler: () => void): Promise<() => void> {
  return listen('focus-search', handler);
}
