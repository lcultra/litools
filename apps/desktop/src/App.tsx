import { createSignal, onCleanup, onMount } from 'solid-js';
import { onNavigate } from './bridge/events';
import type { BuiltinCommandEffect } from './bridge/types';
import { DiagnosticsPage } from './features/diagnostics/DiagnosticsPage';
import { CommandPalette } from './features/palette/CommandPalette';
import { SettingsPage } from './features/settings/SettingsPage';

type ActiveView = 'palette' | 'settings' | 'diagnostics';

export function App() {
  const [activeView, setActiveView] = createSignal<ActiveView>('palette');
  const [darkTheme, setDarkTheme] = createSignal(true);

  onMount(() => {
    const unsubscribe = onNavigate(setActiveView);

    onCleanup(() => {
      void unsubscribe.then((dispose) => dispose());
    });
  });

  function handleCommandEffect(effect: BuiltinCommandEffect) {
    if (effect === 'openSettings') {
      setActiveView('settings');
    }

    if (effect === 'openLogs') {
      setActiveView('diagnostics');
    }

    if (effect === 'toggleTheme') {
      setDarkTheme((current) => !current);
    }

    if (effect === 'reloadIndex') {
      setActiveView('palette');
    }
  }

  return (
    <main
      class="min-h-screen px-4 pt-[12vh] transition-colors"
      classList={{
        'bg-[#111318] text-[#f7f7f8]': darkTheme(),
        'bg-[#f4f5f7] text-[#16181d]': !darkTheme(),
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
        {activeView() === 'settings' ? <SettingsPage /> : null}
        {activeView() === 'diagnostics' ? <DiagnosticsPage /> : null}
      </div>
    </main>
  );
}
