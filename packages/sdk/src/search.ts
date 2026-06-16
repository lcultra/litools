import { invokeSdk } from './runtime';
import type { SearchResult } from './types';

// ---- search API (Phase 3) ----

interface ProviderConfig {
  id: string;
  timeout?: number;
  debounce?: number;
  search: (query: string, signal: AbortSignal) => Promise<SearchResult[]>;
}

/** 注册自定义搜索源，返回 cleanup 函数 */
export async function registerProvider(config: ProviderConfig): Promise<() => Promise<void>> {
  await invokeSdk('sdk_search_register_provider', {
    id: config.id,
    timeout: config.timeout,
  });

  // 注入搜索处理函数
  setupSearchHandler(config);

  return async () => {
    teardownSearchHandler(config.id);
    await invokeSdk('sdk_search_unregister_provider', { id: config.id });
  };
}

/** 注销搜索源 */
export async function unregisterProvider(providerId: string): Promise<void> {
  teardownSearchHandler(providerId);
  await invokeSdk('sdk_search_unregister_provider', { id: providerId });
}

// ── 内部：事件监听管理 ──

const searchHandlers = new Map<string, ProviderConfig>();

function setupSearchHandler(config: ProviderConfig) {
  searchHandlers.set(config.id, config);
}

function teardownSearchHandler(providerId: string) {
  searchHandlers.delete(providerId);
}

// 监听来自 Rust 的搜索请求事件
function handleSearchRequest(event: Event) {
  const detail = (event as CustomEvent)?.detail;
  if (!detail?.providerId || !detail?.requestId) return;

  const handler = searchHandlers.get(detail.providerId);
  if (!handler) return;

  const controller = new AbortController();

  handler
    .search(detail.query, controller.signal)
    .then((results) => {
      if (controller.signal.aborted) return;
      submitResults(detail.requestId, results);
    })
    .catch(() => {
      // 错误返回空结果
      submitResults(detail.requestId, []);
    });
}

async function submitResults(requestId: string, results: SearchResult[]) {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('plugin:litools-sdk|sdk_search_submit', { requestId, results });
  } catch {
    // 忽略提交错误
  }
}

// 注册全局事件监听
if (typeof window !== 'undefined') {
  window.addEventListener('litools:search-request', handleSearchRequest);
}
