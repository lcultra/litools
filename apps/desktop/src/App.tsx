import { useLocation, useNavigate } from '@solidjs/router';
import { createEffect, createSignal, onCleanup, onMount, Show } from 'solid-js';
import { closePluginRuntime, getCurrentSurfaceMetadata, getSettings, hideSurface, updateSurfaceRoute } from './bridge/commands';
import { onNavigate, onSurfaceMetadataChanged } from './bridge/events';
import type { AppSettings, CommandEffect } from './bridge/types';
import { ManagementLayout } from './components/ManagementLayout';
import { DiagnosticsPage } from './features/diagnostics/DiagnosticsPage';
import { CommandPalette } from './features/palette/CommandPalette';
import { PluginManagerPage } from './features/plugins/PluginManagerPage';
import { PluginRuntimeHeaderPage } from './features/plugins/PluginRuntimeHeaderPage';
import { PluginRuntimePage } from './features/plugins/PluginRuntimePage';
import { SettingsPage } from './features/settings/SettingsPage';
import { isDarkThemeValue } from './shared/theme';
import { type AppRoutePath, pluginRuntimeRouteParts, routeForPath } from './views/registry';

export function App() {
    const location = useLocation();
    const navigate = useNavigate();
    const [settings, setSettings] = createSignal<AppSettings | null>(null);
    const [systemDark, setSystemDark] = createSignal(false);

    const [hostWindowLabel, setHostWindowLabel] = createSignal<string | null>(null);
    const [runtimeBreadcrumbs, setRuntimeBreadcrumbs] = createSignal<string[] | null>(null);
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
        if (!hostWindowLabel() || route.id === 'pluginRuntimeHeader' || (route.path === '/' && isDetachedWindow())) {
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
        if (activeRoute().id === 'pluginRuntimeHeader') {
            return;
        }

        if (path === '/' && isDetachedWindow()) {
            return;
        }

        navigate(path);
    }

    function closeManagementPanel() {
        const runtimeRoute = pluginRuntimeRouteParts(activeRoute().path);
        if (runtimeRoute) {
            void closePluginRuntime(runtimeRoute.pluginId, runtimeRoute.commandId);
            safeNavigate('/');
            return;
        }

        if (isDetachedWindow()) {
            void hideSurface();
            return;
        }

        safeNavigate('/');
    }

    function handleSettingsSaved(nextSettings: AppSettings) {
        setSettings(nextSettings);
    }

    function handleCommandEffect(effect: CommandEffect) {
        if (typeof effect === 'object' && 'openPluginView' in effect) {
            safeNavigate(effect.openPluginView.route);
            return;
        }

        if (effect === 'openSettings') {
            safeNavigate('/settings');
        }

        if (effect === 'openDiagnostics') {
            safeNavigate('/diagnostics');
        }

        if (effect === 'openPlugins') {
            safeNavigate('/plugins');
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
            <Show when={activeRoute().id !== 'pluginRuntimeHeader'} fallback={<PluginRuntimeHeaderPage path={activeRoute().path} />}>
                <Show when={!isLauncher()} fallback={<CommandPalette onCommandEffect={handleCommandEffect} />}>
                    <ManagementLayout
                        breadcrumbs={activeRoute().kind === 'runtime' ? (runtimeBreadcrumbs() ?? ['插件', activeRoute().label]) : undefined}
                        isDetached={isDetachedWindow()}
                        mode={activeRoute().kind === 'runtime' ? 'standalone' : 'center'}
                        ownerReady={Boolean(hostWindowLabel())}
                        onOpenLauncher={closeManagementPanel}
                    >
                        <Show when={activeRoute().path === '/settings'}>
                            <SettingsPage onSettingsSaved={handleSettingsSaved} />
                        </Show>
                        <Show when={activeRoute().path === '/diagnostics'}>
                            <DiagnosticsPage />
                        </Show>
                        <Show when={activeRoute().path === '/plugins'}>
                            <PluginManagerPage />
                        </Show>
                        <Show when={activeRoute().kind === 'runtime'}>
                            <PluginRuntimePage onBreadcrumbsChange={setRuntimeBreadcrumbs} path={activeRoute().path} />
                        </Show>
                    </ManagementLayout>
                </Show>
            </Show>
        </main>
    );
}
