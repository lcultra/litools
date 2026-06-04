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
      setError(`保存失败：${String(saveError)}`);
    }
  }

  return (
    <section class="rounded-[20px] border border-border bg-surface p-6 shadow-[var(--shadow-panel)]">
      <div class="flex items-start justify-between gap-4">
        <div>
          <h1 class="m-0 text-2xl font-semibold">设置</h1>
          <p class="m-0 mt-2 text-sm text-muted">配置命令面板运行参数。</p>
        </div>
        <button
          class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-accent-fg disabled:opacity-50"
          disabled={!currentDraft() || saveState() === 'saving'}
          onClick={() => void saveSettings()}
          type="button"
        >
          {saveState() === 'saving' ? '正在保存...' : '保存'}
        </button>
      </div>

      <Show when={currentDraft()} fallback={<p class="mt-6 text-sm text-muted">正在加载设置...</p>}>
        {(settings) => (
          <div class="mt-6 grid gap-4 text-sm">
            <label class="grid gap-2 rounded-xl bg-surface-muted px-4 py-3">
              <span class="text-muted">主题</span>
              <select
                class="rounded-lg border border-border bg-surface px-3 py-2 text-fg outline-none"
                onChange={(event) =>
                  updateDraft((draft) => ({
                    ...draft,
                    theme: event.currentTarget.value,
                  }))
                }
                value={settings().theme}
              >
                <option value="system">跟随系统</option>
                <option value="light">浅色</option>
                <option value="dark">深色</option>
              </select>
            </label>

            <label class="grid gap-2 rounded-xl bg-surface-muted px-4 py-3">
              <span class="text-muted">全局快捷键</span>
              <input
                class="rounded-lg border border-border bg-surface px-3 py-2 text-fg outline-none"
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
              <span class="text-xs text-muted">当前支持：CommandOrControl+Space、Meta+Space、Cmd+Space、Control+Space。</span>
            </label>

            <label class="grid gap-2 rounded-xl bg-surface-muted px-4 py-3">
              <span class="text-muted">结果数量上限</span>
              <input
                class="rounded-lg border border-border bg-surface px-3 py-2 text-fg outline-none"
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

            <SettingRow label="已启用的数据源" value={settings().search.enabled_providers.map(providerLabel).join('，')} />

            <ToggleRow
              checked={settings().window.hide_on_blur}
              label="失焦时隐藏"
              onChange={(checked) =>
                updateDraft((draft) => ({
                  ...draft,
                  window: { ...draft.window, hide_on_blur: checked },
                }))
              }
            />
            <ToggleRow
              checked={settings().window.close_to_tray}
              label="关闭到托盘"
              onChange={(checked) =>
                updateDraft((draft) => ({
                  ...draft,
                  window: { ...draft.window, close_to_tray: checked },
                }))
              }
            />
            <ToggleRow
              checked={settings().window.center_on_show}
              label="显示时居中"
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
        <p class="m-0 mt-4 text-sm text-success">设置已保存</p>
      </Show>
      <Show when={error()}>
        {(message) => <p class="m-0 mt-4 text-sm text-danger">{message()}</p>}
      </Show>
    </section>
  );
}

function providerLabel(provider: string) {
  if (provider === 'commands') {
    return '命令';
  }

  return provider;
}

function SettingRow(props: { label: string; value: string }) {
  return (
    <div class="flex items-center justify-between gap-4 rounded-xl bg-surface-muted px-4 py-3">
      <span class="text-muted">{props.label}</span>
      <span class="font-medium">{props.value}</span>
    </div>
  );
}

function ToggleRow(props: { checked: boolean; label: string; onChange: (checked: boolean) => void }) {
  return (
    <label class="flex items-center justify-between gap-4 rounded-xl bg-surface-muted px-4 py-3">
      <span class="text-muted">{props.label}</span>
      <input checked={props.checked} onChange={(event) => props.onChange(event.currentTarget.checked)} type="checkbox" />
    </label>
  );
}
