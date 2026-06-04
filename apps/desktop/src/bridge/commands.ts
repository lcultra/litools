import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, CommandExecution, DiagnosticsResponse, SearchResult } from './types';

export function search(query: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>('search', { query });
}

export function executeResult(resultId: string, actionId: string): Promise<CommandExecution> {
  return invoke<CommandExecution>('execute_result', { resultId, actionId });
}

export function hideMainWindow(): Promise<void> {
  return invoke<void>('hide_main_window');
}

export function showMainWindow(): Promise<void> {
  return invoke<void>('show_main_window');
}

export function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('get_settings');
}

export function updateSettings(settings: AppSettings): Promise<AppSettings> {
  return invoke<AppSettings>('update_settings', { settings });
}

export function getDiagnostics(): Promise<DiagnosticsResponse> {
  return invoke<DiagnosticsResponse>('get_diagnostics');
}
