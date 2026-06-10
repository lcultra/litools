import { listen } from '@tauri-apps/api/event';
import type { IndexStatus, SurfaceMetadata } from './types';

export function onFocusSearch(handler: () => void): Promise<() => void> {
    return listen('focus-search', handler);
}

export function onIndexStatusChanged(handler: (status: IndexStatus) => void): Promise<() => void> {
    return listen<IndexStatus>('index-status-changed', (event) => handler(event.payload));
}

export function onSurfaceMetadataChanged(handler: (metadata: SurfaceMetadata) => void): Promise<() => void> {
    return listen<SurfaceMetadata>('surface-metadata-changed', (event) => handler(event.payload));
}
