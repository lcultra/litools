import { useLocation, useNavigate } from '@solidjs/router';
import { createEffect, createSignal, onCleanup, onMount, Show } from 'solid-js';
import { closePluginView, getCurrentSurfaceMetadata, getSettings, hideSurface, updateSurfaceRoute } from './bridge/commands';
import { onNavigate, onSurfaceMetadataChanged } from './bridge/events';
import type { AppSettings, CommandEffect, PluginViewState } from './bridge/types';
import { WorkspaceView } from './components/WorkspaceView';
import { Launcher } from './features/launcher/Launcher';
import { TitlebarPage } from './features/titlebar/TitlebarPage';
import { PluginView } from './features/workspace/PluginView';
import { isDarkThemeValue } from './shared/theme';
import { type AppRoutePath, pluginRouteParts, routeForPath } from './views/registry';

export function App() {
    const location = useLocation();
    const navigate = useNavigate();
    const [settings, setSettings] = createSignal<AppSettings | null>(null);
    const [systemDark, setSystemDark] = createSignal(false);

    const [hostWindowLabel, setHostWindowLabel] = createSignal<string | null>(null);
    const [pluginView, setPluginView] = createSignal<PluginViewState | null>(null);
    const isDetachedWindow = () => Boolean(hostWindowLabel() && hostWindowLabel() !== 'main');
    const activeRoute = () => routeForPath(location.pathname);
    const isLauncher = () => activeRoute().kind === 'launcher';

    onMount(() => {
        void refreshSettings();
        void restoreSurfaceHost();

        const media = window.matchMedia('(prefers-color-scheme: dark)');
        setSystemDark(media.matches);
        const handleSystemTheme = (event: MediaQueryListEvent) => setSystemDark(event.matches);
        media.addEventListener('change', handleSystemTheme);

        const unsubscribe = onNavigate((path) => safeNavigate(path));
        const unsubscribeSurfaceMetadata = onSurfaceMetadataChanged((metadata) => {
            setHostWindowLabel(metadata.hostWindowLabel);
        });
        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.key !== 'Escape' || isLauncher()) {
                return;
            }

            event.preventDefault();
            closeCurrentView();
        };
        window.addEventListener('keydown', handleKeyDown);

        function preventContextMenu(event: MouseEvent) {
            event.preventDefault();
        }
        window.addEventListener('contextmenu', preventContextMenu);

        onCleanup(() => {
            media.removeEventListener('change', handleSystemTheme);
            window.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('contextmenu', preventContextMenu);
            void unsubscribe.then((dispose) => dispose());
            void unsubscribeSurfaceMetadata.then((dispose) => dispose());
        });
    });

    createEffect(() => {
        document.documentElement.classList.toggle('dark', isDarkTheme());
    });

    createEffect(() => {
        const route = activeRoute();
        if (!hostWindowLabel() || route.id === 'titlebar' || (route.path === '/' && isDetachedWindow())) {
            return;
        }

        void updateSurfaceRoute(route.path);
    });

    async function refreshSettings() {
        setSettings(await getSettings());
    }

    async function restoreSurfaceHost() {
        const metadata = await getCurrentSurfaceMetadata();
        setHostWindowLabel(metadata?.hostWindowLabel ?? 'main');
    }

    function isDarkTheme() {
        return isDarkThemeValue(settings()?.theme, systemDark());
    }

    function safeNavigate(path: AppRoutePath) {
        if (activeRoute().id === 'titlebar') {
            return;
        }

        if (path === '/' && isDetachedWindow()) {
            return;
        }

        navigate(path);
    }

    function closeCurrentView() {
        const parts = pluginRouteParts(activeRoute().path);
        if (parts) {
            void closePluginView(parts.pluginId, parts.commandId);
            safeNavigate('/');
            return;
        }

        if (isDetachedWindow()) {
            void hideSurface();
            return;
        }

        safeNavigate('/');
    }

    function handleCommandEffect(effect: CommandEffect) {
        if (typeof effect === 'object' && 'openPluginView' in effect) {
            safeNavigate(effect.openPluginView.route);
            return;
        }

        if (effect === 'toggleTheme') {
            void refreshSettings();
        }

        if (effect === 'reloadIndex') {
            safeNavigate('/');
        }
    }

    return (
        <main class="h-screen overflow-hidden text-fg transition-colors">
            <Show when={activeRoute().id !== 'titlebar'} fallback={<TitlebarPage path={activeRoute().path} />}>
                <Show when={!isLauncher()} fallback={<Launcher onCommandEffect={handleCommandEffect} />}>
                    <WorkspaceView isDetached={isDetachedWindow()} onClose={closeCurrentView} ownerReady={Boolean(hostWindowLabel())} pluginView={pluginView()}>
                        <Show when={activeRoute().kind === 'plugin'}>
                            <PluginView onStateChange={setPluginView} path={activeRoute().path} />
                        </Show>
                    </WorkspaceView>
                </Show>
            </Show>
        </main>
    );
}
