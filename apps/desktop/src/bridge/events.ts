import { listen } from '@tauri-apps/api/event';

export type NavigationView = 'palette' | 'settings' | 'diagnostics';

export function onFocusSearch(handler: () => void): Promise<() => void> {
    return listen('focus-search', handler);
}

export function onNavigate(handler: (view: NavigationView) => void): Promise<() => void> {
    return listen<string>('navigate', (event) => {
        if (event.payload === 'palette' || event.payload === 'settings' || event.payload === 'diagnostics') {
            handler(event.payload);
        }
    });
}
