import { HashRouter, Route } from '@solidjs/router';
import { createEffect, onCleanup, onMount } from 'solid-js';
import { LauncherPage } from './features/launcher/LauncherPage';
import { WorkspacePage } from './features/workspace/WorkspacePage';
import { useAppEvents } from './hooks/useAppEvents';
import { PLUGIN_ROUTE_PATTERN } from './shared/routes';
import { settings } from './shared/store';
import { isDarkThemeValue } from './shared/theme';

export function App() {
    // Tauri events → shared store
    useAppEvents();

    // Theme
    createEffect(() => {
        const mq = window.matchMedia('(prefers-color-scheme: dark)');
        const update = () => document.documentElement.classList.toggle('dark', isDarkThemeValue(settings()?.theme, mq.matches));
        update();
        const handler = () => update();
        mq.addEventListener('change', handler);
        onCleanup(() => mq.removeEventListener('change', handler));
    });

    // Global: disable native context menu on all webviews
    onMount(() => {
        const preventCtx = (e: MouseEvent) => e.preventDefault();
        window.addEventListener('contextmenu', preventCtx);
        onCleanup(() => window.removeEventListener('contextmenu', preventCtx));
    });

    return (
        <main class="h-screen overflow-hidden text-fg transition-colors">
            <HashRouter>
                <Route path="/" component={LauncherPage} />
                <Route path={PLUGIN_ROUTE_PATTERN} component={WorkspacePage} />
            </HashRouter>
        </main>
    );
}
