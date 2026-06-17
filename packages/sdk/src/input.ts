import { invokeSdk } from './runtime';
import type { Detection } from './types';

// ── input API (Phase 4D) ──

interface DetectorConfig {
  id: string;
  featureKind?: string;
  timeout?: number;
  detect: (input: string) => Detection | null | Promise<Detection | null>;
}

/** 注册自定义输入检测器，返回 cleanup 函数 */
export async function registerDetector(config: DetectorConfig): Promise<() => Promise<void>> {
  await invokeSdk('sdk_input_register_detector', {
    id: config.id,
    featureKind: config.featureKind,
    timeout: config.timeout,
  });

  setupDetectionHandler(config);

  return async () => {
    teardownDetectionHandler(config.id);
    await invokeSdk('sdk_input_unregister_detector', { id: config.id });
  };
}

/** 注销输入检测器 */
export async function unregisterDetector(detectorId: string): Promise<void> {
  teardownDetectionHandler(detectorId);
  await invokeSdk('sdk_input_unregister_detector', { id: detectorId });
}

// ── 内部：事件监听管理 ──

const detectionHandlers = new Map<string, DetectorConfig>();

function setupDetectionHandler(config: DetectorConfig) {
  detectionHandlers.set(config.id, config);
}

function teardownDetectionHandler(detectorId: string) {
  detectionHandlers.delete(detectorId);
}

// 监听来自 Rust 的检测请求事件
async function handleDetectionRequest(event: Event) {
  const detail = (event as CustomEvent)?.detail;
  if (!detail?.detectorId || !detail?.requestId || !detail?.input) return;

  const handler = detectionHandlers.get(detail.localDetectorId ?? detail.detectorId);
  if (!handler) return;

  try {
    const detection = await handler.detect(detail.input);
    submitDetection(detail.requestId, detection);
  } catch {
    submitDetection(detail.requestId, null);
  }
}

async function submitDetection(requestId: string, detection: Detection | null) {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('plugin:litools-sdk|sdk_detection_submit', { requestId, detection });
  } catch {
    // 忽略提交错误
  }
}

// 注册全局事件监听
if (typeof window !== 'undefined') {
  window.addEventListener('litools:detection-request', handleDetectionRequest);
}
