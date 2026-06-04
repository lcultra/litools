import { createEffect, createSignal, For, onCleanup, onMount, Show } from 'solid-js';
import { getSettings } from './bridge/commands';
import { onNavigate } from './bridge/events';
import type { AppSettings, BuiltinCommandEffect } from './bridge/types';
import { DiagnosticsPage } from './features/diagnostics/DiagnosticsPage';
import { CommandPalette } from './features/palette/CommandPalette';
import { PluginManagerPage } from './features/plugins/PluginManagerPage';
import { SettingsPage } from './features/settings/SettingsPage';
import { type AppViewId, secondaryViewNavItems } from './views/registry';

export function App() {
    const [activeView, setActiveView] = createSignal<AppViewId>('palette');
    const [settings, setSettings] = createSignal<AppSettings | null>(null);
    const [systemDark, setSystemDark] = createSignal(false);

    onMount(() => {
        void refreshSettings();

        const media = window.matchMedia('(prefers-color-scheme: dark)');
        setSystemDark(media.matches);
        const handleSystemTheme = (event: MediaQueryListEvent) => setSystemDark(event.matches);
        media.addEventListener('change', handleSystemTheme);

        const unsubscribe = onNavigate(setActiveView);

        onCleanup(() => {
            media.removeEventListener('change', handleSystemTheme);
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
        const theme = settings()?.theme;
        return theme === 'dark' || (theme === 'system' && systemDark());
    }

    function handleSettingsSaved(nextSettings: AppSettings) {
        setSettings(nextSettings);
    }

    function handleCommandEffect(effect: BuiltinCommandEffect) {
        if (effect === 'openSettings') {
            setActiveView('settings');
        }

        if (effect === 'openLogs') {
            setActiveView('diagnostics');
        }

        if (effect === 'toggleTheme') {
            void refreshSettings();
        }

        if (effect === 'reloadIndex') {
            setActiveView('palette');
        }
    }

    return (
        <main class="h-screen overflow-hidden text-fg transition-colors">
            <div class="grid w-full">
                <Show when={activeView() !== 'palette'}>
                    <nav class="flex items-center justify-between rounded-2xl border border-border bg-surface px-3 py-2 text-sm text-muted shadow-[var(--shadow-panel)]">
                        <button class="rounded-lg px-3 py-2 font-semibold text-fg hover:bg-surface-muted" onClick={() => setActiveView('palette')} type="button">
                            litools
                        </button>
                        <div class="flex gap-1">
                            <For each={secondaryViewNavItems}>
                                {(item) => (
                                    <button
                                        class="rounded-lg px-3 py-2 outline-none transition-colors hover:bg-surface-muted/60 focus-visible:bg-surface-muted/60"
                                        classList={{ 'bg-surface-muted text-fg': activeView() === item.id }}
                                        onClick={() => setActiveView(item.id)}
                                        type="button"
                                    >
                                        {item.label}
                                    </button>
                                )}
                            </For>
                        </div>
                    </nav>
                </Show>

                {activeView() === 'palette' ? <CommandPalette onCommandEffect={handleCommandEffect} /> : null}
                {activeView() === 'settings' ? <SettingsPage onSettingsSaved={handleSettingsSaved} /> : null}
                {activeView() === 'diagnostics' ? <DiagnosticsPage /> : null}
                {activeView() === 'plugins' ? <PluginManagerPage /> : null}
            </div>
        </main>
    );
}
