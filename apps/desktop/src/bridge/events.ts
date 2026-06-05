import { listen } from '@tauri-apps/api/event';
import { type AppRoutePath, pathForNavigationPayload } from '../views/registry';
import type { IndexStatus } from './types';

export function onFocusSearch(handler: () => void): Promise<() => void> {
    return listen('focus-search', handler);
}

export function onNavigate(handler: (path: AppRoutePath) => void): Promise<() => void> {
    return listen<string>('navigate', (event) => {
        const path = pathForNavigationPayload(event.payload);

        if (path) {
            handler(path);
        }
    });
}

export function onIndexStatusChanged(handler: (status: IndexStatus) => void): Promise<() => void> {
    return listen<IndexStatus>('index-status-changed', (event) => handler(event.payload));
}
