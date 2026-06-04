import { invoke } from '@tauri-apps/api/core';
import type { SearchResult } from './types';

export function search(query: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>('search', { query });
}

export function executeResult(resultId: string, actionId: string): Promise<string> {
  return invoke<string>('execute_result', { resultId, actionId });
}
