import { createListCollection, Select } from '@ark-ui/solid/select';
import { Switch } from '@ark-ui/solid/switch';
import { createEffect, createResource, createSignal, For, Show } from 'solid-js';
import { Portal } from 'solid-js/web';
import { getSettings, updateSettings } from '../../bridge/commands';
import type { AppSettings } from '../../bridge/types';
import { InfoRow } from '../../components/InfoRow';
import { PageHeader } from '../../components/PageHeader';
import { Panel } from '../../components/Panel';
import { providerListLabel } from '../../shared/labels';
import { isThemeValue, type ThemeOption, themeOptions } from '../../shared/theme';

type SettingsPageProps = {
    onSettingsSaved: (settings: AppSettings) => void;
};

type SaveState = 'idle' | 'saving' | 'saved' | 'error';

const themeCollection = createListCollection<ThemeOption>({
    items: themeOptions,
    itemToString: (item) => item.label,
    itemToValue: (item) => item.value,
});

export function SettingsPage(props: SettingsPageProps) {
    const [settings] = createResource(getSettings);
    const [draft, setDraft] = createSignal<AppSettings | null>(null);
    const [saveState, setSaveState] = createSignal<SaveState>('idle');
    const [error, setError] = createSignal<string | null>(null);

    createEffect(() => {
        const loadedSettings = settings();

        if (loadedSettings && !draft()) {
            setDraft(structuredClone(loadedSettings));
        }
    });

    function currentDraft() {
        return draft();
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
        <Panel>
            <PageHeader
                action={
                    <button
                        class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-accent-fg disabled:opacity-50"
                        disabled={!currentDraft() || saveState() === 'saving'}
                        onClick={() => void saveSettings()}
                        type="button"
                    >
                        {saveState() === 'saving' ? '正在保存...' : '保存'}
                    </button>
                }
                description="配置命令面板运行参数。"
                title="设置"
            />

            <Show when={currentDraft()} fallback={<p class="mt-6 text-sm text-muted">正在加载设置...</p>}>
                {(settings) => (
                    <div class="mt-6 grid gap-4 text-sm">
                        <Select.Root
                            class="grid gap-2 rounded-xl bg-surface-muted px-4 py-3"
                            collection={themeCollection}
                            onValueChange={(details) => {
                                const [theme] = details.value;

                                if (!theme || !isThemeValue(theme)) {
                                    return;
                                }

                                updateDraft((draft) => ({
                                    ...draft,
                                    theme,
                                }));
                            }}
                            value={[settings().theme]}
                        >
                            <Select.Label class="text-muted">主题</Select.Label>
                            <Select.Control>
                                <Select.Trigger class="flex w-full items-center justify-between rounded-lg border border-border bg-surface px-3 py-2 text-fg outline-none">
                                    <Select.ValueText />
                                    <Select.Indicator class="text-muted">⌄</Select.Indicator>
                                </Select.Trigger>
                            </Select.Control>
                            <Portal>
                                <Select.Positioner>
                                    <Select.Content class="z-50 mt-1 min-w-[var(--reference-width)] rounded-lg border border-border bg-surface p-1 text-fg shadow-[var(--shadow-panel)]">
                                        <For each={themeCollection.items}>
                                            {(item) => (
                                                <Select.Item
                                                    class="cursor-pointer rounded-md px-3 py-2 outline-none data-[highlighted]:bg-surface-muted data-[state=checked]:font-semibold"
                                                    item={item}
                                                >
                                                    <Select.ItemText>{item.label}</Select.ItemText>
                                                    <Select.ItemIndicator class="float-right text-muted">✓</Select.ItemIndicator>
                                                </Select.Item>
                                            )}
                                        </For>
                                    </Select.Content>
                                </Select.Positioner>
                            </Portal>
                            <Select.HiddenSelect />
                        </Select.Root>

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

                        <InfoRow label="已启用的数据源" value={providerListLabel(settings().search.enabled_providers)} />

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
            <Show when={error()}>{(message) => <p class="m-0 mt-4 text-sm text-danger">{message()}</p>}</Show>
        </Panel>
    );
}

function ToggleRow(props: { checked: boolean; label: string; onChange: (checked: boolean) => void }) {
    return (
        <Switch.Root
            checked={props.checked}
            class="flex items-center justify-between gap-4 rounded-xl bg-surface-muted px-4 py-3"
            onCheckedChange={(details) => props.onChange(details.checked)}
        >
            <Switch.Label class="text-muted">{props.label}</Switch.Label>
            <Switch.Control class="relative h-6 w-11 rounded-full bg-border transition-colors data-[state=checked]:bg-accent">
                <Switch.Thumb class="block size-5 translate-x-0.5 rounded-full bg-surface transition-transform data-[state=checked]:translate-x-5" />
            </Switch.Control>
            <Switch.HiddenInput />
        </Switch.Root>
    );
}
