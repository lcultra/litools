import { invokeSdk } from './runtime';
import type { DiagnosticsResponse } from './types';

// ---- diagnostics API ----

export function get(): Promise<DiagnosticsResponse> {
  return invokeSdk('sdk_diagnostics_get');
}
