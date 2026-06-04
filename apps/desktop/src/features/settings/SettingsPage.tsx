import { Show, createResource, createSignal } from 'solid-js';
import { getSettings, updateSettings } from '../../bridge/commands';
import type { AppSettings } from '../../bridge/types';

type SettingsPageProps = {
  onSettingsSaved: (settings: AppSettings) => void;
};

type SaveState = 'idle' | 'saving' | 'saved' | 'error';

export function SettingsPage(props: SettingsPageProps) {
  const [settings] = createResource(getSettings);
  const [draft, setDraft] = createSignal<AppSettings | null>(null);
  const [saveState, setSaveState] = createSignal<SaveState>('idle');
  const [error, setError] = createSignal<string | null>(null);

  function currentDraft() {
    const nextDraft = draft();
    const loadedSettings = settings();

    if (nextDraft) {
      return nextDraft;
    }

    if (loadedSettings) {
      setDraft(structuredClone(loadedSettings));
      return loadedSettings;
    }

    return null;
  }

  function updateDraft(update: (settings: AppSettings) => AppSettings) {
    const current = currentDraft();

    if (!current) {
      return;
    }

    setSaveState('idle');
    setDraft(update(structuredClone(current)));
  }

  async function saveSettings() {
    const current = currentDraft();

    if (!current) {
      return;
    }

    setSaveState('saving');
    setError(null);

    try {
      const saved = await updateSettings(current);
      setDraft(saved);
      props.onSettingsSaved(saved);
      setSaveState('saved');
    } catch (saveError) {
      setSaveState('error');
      setError(String(saveError));
    }
  }

  return (
    <section class="rounded-[20px] border border-current/10 bg-current/5 p-6 shadow-[0_24px_80px_rgba(0,0,0,0.18)]">
      <div class="flex items-start justify-between gap-4">
        <div>
          <h1 class="m-0 text-2xl font-semibold">Settings</h1>
          <p class="m-0 mt-2 text-sm text-current/60">Configure the Phase 1 command palette runtime.</p>
        </div>
        <button
          class="rounded-lg bg-current px-4 py-2 text-sm font-semibold text-white disabled:opacity-50 dark:text-[#111318]"
          disabled={!currentDraft() || saveState() === 'saving'}
          onClick={() => void saveSettings()}
          type="button"
        >
          {saveState() === 'saving' ? 'Saving...' : 'Save'}
        </button>
      </div>

      <Show when={currentDraft()} fallback={<p class="mt-6 text-sm text-current/60">Loading settings...</p>}>
        {(settings) => (
          <div class="mt-6 grid gap-4 text-sm">
            <label class="grid gap-2 rounded-xl bg-current/5 px-4 py-3">
              <span class="text-current/60">Theme</span>
              <select
                class="rounded-lg border border-current/10 bg-transparent px-3 py-2 outline-none"
                onChange={(event) =>
                  updateDraft((draft) => ({
                    ...draft,
                    theme: event.currentTarget.value,
                  }))
                }
                value={settings().theme}
              >
                <option value="system">System</option>
                <option value="light">Light</option>
                <option value="dark">Dark</option>
              </select>
            </label>

            <label class="grid gap-2 rounded-xl bg-current/5 px-4 py-3">
              <span class="text-current/60">Global hotkey</span>
              <input
                class="rounded-lg border border-current/10 bg-transparent px-3 py-2 outline-none"
                onInput={(event) =>
                  updateDraft((draft) => ({
                    ...draft,
                    palette: {
                      ...draft.palette,
                      global_hotkey: event.currentTarget.value,
                    },
                  }))
                }
                value={settings().palette.global_hotkey}
              />
              <span class="text-xs text-current/50">Supported now: CommandOrControl+Space, Meta+Space, Cmd+Space, Control+Space.</span>
            </label>

            <label class="grid gap-2 rounded-xl bg-current/5 px-4 py-3">
              <span class="text-current/60">Result limit</span>
              <input
                class="rounded-lg border border-current/10 bg-transparent px-3 py-2 outline-none"
                min="1"
                max="50"
                onInput={(event) =>
                  updateDraft((draft) => ({
                    ...draft,
                    palette: {
                      ...draft.palette,
                      result_limit: Number(event.currentTarget.value),
                    },
                  }))
                }
                type="number"
                value={settings().palette.result_limit}
              />
            </label>

            <SettingRow label="Enabled providers" value={settings().search.enabled_providers.join(', ')} />

            <ToggleRow
              checked={settings().window.hide_on_blur}
              label="Hide on blur"
              onChange={(checked) =>
                updateDraft((draft) => ({
                  ...draft,
                  window: { ...draft.window, hide_on_blur: checked },
                }))
              }
            />
            <ToggleRow
              checked={settings().window.close_to_tray}
              label="Close to tray"
              onChange={(checked) =>
                updateDraft((draft) => ({
                  ...draft,
                  window: { ...draft.window, close_to_tray: checked },
                }))
              }
            />
            <ToggleRow
              checked={settings().window.center_on_show}
              label="Center on show"
              onChange={(checked) =>
                updateDraft((draft) => ({
                  ...draft,
                  window: { ...draft.window, center_on_show: checked },
                }))
              }
            />
          </div>
        )}
      </Show>

      <Show when={saveState() === 'saved'}>
        <p class="m-0 mt-4 text-sm text-emerald-500">Settings saved.</p>
      </Show>
      <Show when={error()}>
        {(message) => <p class="m-0 mt-4 text-sm text-red-500">{message()}</p>}
      </Show>
    </section>
  );
}

function SettingRow(props: { label: string; value: string }) {
  return (
    <div class="flex items-center justify-between gap-4 rounded-xl bg-current/5 px-4 py-3">
      <span class="text-current/60">{props.label}</span>
      <span class="font-medium">{props.value}</span>
    </div>
  );
}

function ToggleRow(props: { checked: boolean; label: string; onChange: (checked: boolean) => void }) {
  return (
    <label class="flex items-center justify-between gap-4 rounded-xl bg-current/5 px-4 py-3">
      <span class="text-current/60">{props.label}</span>
      <input checked={props.checked} onChange={(event) => props.onChange(event.currentTarget.checked)} type="checkbox" />
    </label>
  );
}
