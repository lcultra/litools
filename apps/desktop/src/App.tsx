import { useLocation, useNavigate } from '@solidjs/router';
import { createEffect, createSignal, onCleanup, onMount, Show } from 'solid-js';
import { getSettings } from './bridge/commands';
import { onNavigate } from './bridge/events';
import type { AppSettings, BuiltinCommandEffect } from './bridge/types';
import { ManagementLayout } from './components/ManagementLayout';
import { DiagnosticsPage } from './features/diagnostics/DiagnosticsPage';
import { CommandPalette } from './features/palette/CommandPalette';
import { PluginManagerPage } from './features/plugins/PluginManagerPage';
import { SettingsPage } from './features/settings/SettingsPage';
import { isDarkThemeValue } from './shared/theme';
import { routeForPath } from './views/registry';

export function App() {
    const location = useLocation();
    const navigate = useNavigate();
    const [settings, setSettings] = createSignal<AppSettings | null>(null);
    const [systemDark, setSystemDark] = createSignal(false);

    const activeRoute = () => routeForPath(location.pathname);
    const isLauncher = () => activeRoute().kind === 'launcher';

    onMount(() => {
        void refreshSettings();

        const media = window.matchMedia('(prefers-color-scheme: dark)');
        setSystemDark(media.matches);
        const handleSystemTheme = (event: MediaQueryListEvent) => setSystemDark(event.matches);
        media.addEventListener('change', handleSystemTheme);

        const unsubscribe = onNavigate((path) => navigate(path));
        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.key !== 'Escape' || isLauncher()) {
                return;
            }

            event.preventDefault();
            navigate('/');
        };
        window.addEventListener('keydown', handleKeyDown);

        onCleanup(() => {
            media.removeEventListener('change', handleSystemTheme);
            window.removeEventListener('keydown', handleKeyDown);
            void unsubscribe.then((dispose) => dispose());
        });
    });

    createEffect(() => {
        document.documentElement.classList.toggle('dark', isDarkTheme());
    });

    async function refreshSettings() {
        setSettings(await getSettings());
    }

    function isDarkTheme() {
        return isDarkThemeValue(settings()?.theme, systemDark());
    }

    function openLauncher() {
        navigate('/');
    }

    function handleSettingsSaved(nextSettings: AppSettings) {
        setSettings(nextSettings);
    }

    function handleCommandEffect(effect: BuiltinCommandEffect) {
        if (effect === 'openSettings') {
            navigate('/settings');
        }

        if (effect === 'openLogs') {
            navigate('/diagnostics');
        }

        if (effect === 'toggleTheme') {
            void refreshSettings();
        }

        if (effect === 'reloadIndex') {
            navigate('/');
        }
    }

    return (
        <main class="h-screen overflow-hidden text-fg transition-colors">
            <Show when={!isLauncher()} fallback={<CommandPalette onCommandEffect={handleCommandEffect} />}>
                <ManagementLayout onOpenLauncher={openLauncher}>
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
