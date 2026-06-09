import { listen } from '@tauri-apps/api/event';
import { isPluginRoutePath } from '../views/registry';
import type { IndexStatus, SurfaceMetadata } from './types';

export function onFocusSearch(handler: () => void): Promise<() => void> {
    return listen('focus-search', handler);
}

export function onNavigate(handler: (path: string) => void): Promise<() => void> {
    return listen<string>('navigate', (event) => {
        const path = event.payload;
        if (path === '/' || isPluginRoutePath(path)) {
            handler(path);
        }
    });
}

export function onIndexStatusChanged(handler: (status: IndexStatus) => void): Promise<() => void> {
    return listen<IndexStatus>('index-status-changed', (event) => handler(event.payload));
}

export function onSurfaceMetadataChanged(handler: (metadata: SurfaceMetadata) => void): Promise<() => void> {
    return listen<SurfaceMetadata>('surface-metadata-changed', (event) => handler(event.payload));
}
