import { useLocation, useNavigate } from '@solidjs/router';
import { createEffect, createSignal, onCleanup, onMount, Show } from 'solid-js';
import { getCurrentWindowMetadata, getSettings, hideWindow, updateSurfaceRoute } from './bridge/commands';
import { onNavigate, onSurfaceMetadataChanged } from './bridge/events';
import type { AppSettings, BuiltinCommandEffect } from './bridge/types';
import { ManagementLayout } from './components/ManagementLayout';
import { DiagnosticsPage } from './features/diagnostics/DiagnosticsPage';
import { CommandPalette } from './features/palette/CommandPalette';
import { PluginManagerPage } from './features/plugins/PluginManagerPage';
import { SettingsPage } from './features/settings/SettingsPage';
import { isDarkThemeValue } from './shared/theme';
import { type AppRoutePath, routeForPath } from './views/registry';

export function App() {
    const location = useLocation();
    const navigate = useNavigate();
    const [settings, setSettings] = createSignal<AppSettings | null>(null);
    const [systemDark, setSystemDark] = createSignal(false);

    const [ownerWindowLabel, setOwnerWindowLabel] = createSignal<string | null>(null);
    const isDetachedWindow = () => Boolean(ownerWindowLabel() && ownerWindowLabel() !== 'main');
    const activeRoute = () => routeForPath(location.pathname);
    const isLauncher = () => activeRoute().kind === 'launcher';

    onMount(() => {
        void refreshSettings();
        void restoreSurfaceOwner();

        const media = window.matchMedia('(prefers-color-scheme: dark)');
        setSystemDark(media.matches);
        const handleSystemTheme = (event: MediaQueryListEvent) => setSystemDark(event.matches);
        media.addEventListener('change', handleSystemTheme);

        const unsubscribe = onNavigate((path) => safeNavigate(path));
        const unsubscribeSurfaceMetadata = onSurfaceMetadataChanged((metadata) => {
            setOwnerWindowLabel(metadata.ownerWindowLabel);
        });
        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.key !== 'Escape' || isLauncher()) {
                return;
            }

            event.preventDefault();
            closeManagementPanel();
        };
        window.addEventListener('keydown', handleKeyDown);

        onCleanup(() => {
            media.removeEventListener('change', handleSystemTheme);
            window.removeEventListener('keydown', handleKeyDown);
            void unsubscribe.then((dispose) => dispose());
            void unsubscribeSurfaceMetadata.then((dispose) => dispose());
        });
    });

    createEffect(() => {
        document.documentElement.classList.toggle('dark', isDarkTheme());
    });

    createEffect(() => {
        const route = activeRoute();
        if (!ownerWindowLabel() || (route.path === '/' && isDetachedWindow())) {
            return;
        }

        void updateSurfaceRoute(route.path);
    });

    async function refreshSettings() {
        setSettings(await getSettings());
    }

    async function restoreSurfaceOwner() {
        const metadata = await getCurrentWindowMetadata();
        setOwnerWindowLabel(metadata?.ownerWindowLabel ?? 'main');
    }

    function isDarkTheme() {
        return isDarkThemeValue(settings()?.theme, systemDark());
    }

    function safeNavigate(path: AppRoutePath) {
        if (path === '/' && isDetachedWindow()) {
            return;
        }

        navigate(path);
    }

    function closeManagementPanel() {
        if (isDetachedWindow()) {
            void hideWindow();
            return;
        }

        safeNavigate('/');
    }

    function handleSettingsSaved(nextSettings: AppSettings) {
        setSettings(nextSettings);
    }

    function handleCommandEffect(effect: BuiltinCommandEffect) {
        if (effect === 'openSettings') {
            safeNavigate('/settings');
        }

        if (effect === 'openLogs') {
            safeNavigate('/diagnostics');
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
            <Show when={!isLauncher()} fallback={<CommandPalette onCommandEffect={handleCommandEffect} />}>
                <ManagementLayout isDetached={isDetachedWindow()} ownerReady={Boolean(ownerWindowLabel())} onOpenLauncher={closeManagementPanel}>
                    <Show when={activeRoute().path === '/settings'}>
                        <SettingsPage onSettingsSaved={handleSettingsSaved} />
                    </Show>
                    <Show when={activeRoute().path === '/diagnostics'}>
                        <DiagnosticsPage />
                    </Show>
                    <Show when={activeRoute().path === '/plugins'}>
                        <PluginManagerPage />
                    </Show>
                </ManagementLayout>
            </Show>
        </main>
    );
}
