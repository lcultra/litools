import { createRoot, createSignal } from 'solid-js';

// Module-level signals — survive component remounts via createRoot.
const [_hostWindowLabel, _setHostWindowLabel] = createRoot(() => createSignal<string | null>(null));

export const hostWindowLabel = () => _hostWindowLabel();
export const setHostWindowLabel = (v: string | null) => _setHostWindowLabel(v);
export const isDetachedWindow = () => Boolean(hostWindowLabel() && hostWindowLabel() !== 'main');
