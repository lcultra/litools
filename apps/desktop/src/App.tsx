import { createEffect, createSignal, onCleanup, onMount } from 'solid-js';
import { getSettings } from './bridge/commands';
import { onNavigate } from './bridge/events';
import type { AppSettings, BuiltinCommandEffect } from './bridge/types';
import { DiagnosticsPage } from './features/diagnostics/DiagnosticsPage';
import { CommandPalette } from './features/palette/CommandPalette';
import { SettingsPage } from './features/settings/SettingsPage';

type ActiveView = 'palette' | 'settings' | 'diagnostics';

export function App() {
  const [activeView, setActiveView] = createSignal<ActiveView>('palette');
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
    <main
      class="min-h-screen px-4 pt-[12vh] transition-colors"
      classList={{
        'bg-[#111318] text-[#f7f7f8]': isDarkTheme(),
        'bg-[#f4f5f7] text-[#16181d]': !isDarkTheme(),
      }}
    >
      <div class="mx-auto grid w-[min(720px,calc(100vw-32px))] gap-4">
        <nav class="flex items-center justify-between text-sm text-current/60">
          <button class="rounded-lg px-3 py-2 hover:bg-current/10" onClick={() => setActiveView('palette')} type="button">
            litools
          </button>
          <div class="flex gap-2">
            <button class="rounded-lg px-3 py-2 hover:bg-current/10" onClick={() => setActiveView('settings')} type="button">
              Settings
            </button>
            <button class="rounded-lg px-3 py-2 hover:bg-current/10" onClick={() => setActiveView('diagnostics')} type="button">
              Diagnostics
            </button>
          </div>
        </nav>

        {activeView() === 'palette' ? <CommandPalette onCommandEffect={handleCommandEffect} /> : null}
        {activeView() === 'settings' ? <SettingsPage onSettingsSaved={handleSettingsSaved} /> : null}
        {activeView() === 'diagnostics' ? <DiagnosticsPage /> : null}
      </div>
    </main>
  );
}
