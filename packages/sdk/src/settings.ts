import { invokeSdk } from './runtime';
import type { AppSettings } from './types';

// ---- settings API ----

export function get(): Promise<AppSettings> {
  return invokeSdk('sdk_settings_get');
}

export function update(settings: AppSettings): Promise<AppSettings> {
  return invokeSdk('sdk_settings_update', { settings });
}
