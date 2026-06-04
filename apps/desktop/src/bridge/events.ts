import { listen } from '@tauri-apps/api/event';
import { type AppViewId, isAppViewId } from '../views/registry';

export function onFocusSearch(handler: () => void): Promise<() => void> {
    return listen('focus-search', handler);
}

export function onNavigate(handler: (view: AppViewId) => void): Promise<() => void> {
    return listen<string>('navigate', (event) => {
        if (isAppViewId(event.payload)) {
            handler(event.payload);
        }
    });
}
