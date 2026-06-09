import { createRoot, createSignal } from 'solid-js';
import type { AppSettings } from '../bridge/types';

// Module-level signals — survive component remounts via createRoot.
const [_hostWindowLabel, _setHostWindowLabel] = createRoot(() => createSignal<string | null>(null));
const [_settings, _setSettings] = createRoot(() => createSignal<AppSettings | null>(null));

export const hostWindowLabel = () => _hostWindowLabel();
export const setHostWindowLabel = (v: string | null) => _setHostWindowLabel(v);
export const isDetachedWindow = () => Boolean(hostWindowLabel() && hostWindowLabel() !== 'main');

export const settings = () => _settings();
export const setSettings = (v: AppSettings | null) => _setSettings(v);
