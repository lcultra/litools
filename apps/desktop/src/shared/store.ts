import { createRoot, createSignal } from 'solid-js';
import { getBaseInfo } from '../bridge/commands';
import type { BaseInfo } from '../bridge/types';

// Module-level signals — survive component remounts via createRoot.
const [_hostWindowLabel, _setHostWindowLabel] = createRoot(() => createSignal<string | null>(null));
const [_baseInfo, _setBaseInfo] = createRoot(() => createSignal<BaseInfo | null>(null));

// 启动时调用一次，后续全局共享
let _initialized = false;
export function initBaseInfo() {
    if (_initialized) return;
    _initialized = true;
    void getBaseInfo().then((info) => _setBaseInfo(info));
}

export const hostWindowLabel = () => _hostWindowLabel();
export const setHostWindowLabel = (v: string | null) => _setHostWindowLabel(v);

export const baseInfo = () => _baseInfo();

export const isDetachedWindow = () => {
    const info = _baseInfo();
    const label = _hostWindowLabel();
    if (!info || !label) return false;
    return label !== info.mainWindowLabel;
};
