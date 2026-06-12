import { HashRouter, Route } from '@solidjs/router';
import { onCleanup, onMount } from 'solid-js';
import { LauncherPage } from './features/launcher/LauncherPage';
import { WorkspacePage } from './features/workspace/WorkspacePage';
import { useAppEvents } from './hooks/useAppEvents';
import { PLUGIN_ROUTE_PATTERN } from './shared/routes';

export function App() {
    // Tauri events → shared store
    useAppEvents();

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
