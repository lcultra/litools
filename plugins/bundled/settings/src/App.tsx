import { createSignal, onMount } from 'solid-js';
import { settingsGet, settingsUpdate, ready } from '@litools/plugin-sdk';
import { PluginLayout, SegmentedControl, Input } from '@litools/plugin-ui';
import type { SegmentedOption } from '@litools/plugin-ui';
import type { ThemeValue, AppSettings } from '@litools/plugin-core';

const themeOptions: SegmentedOption<ThemeValue>[] = [
    { value: 'system', label: '跟随系统' },
    { value: 'light', label: '浅色' },
    { value: 'dark', label: '深色' },
];

function themeLabel(theme: string): string {
    switch (theme) {
        case 'dark': return '深色';
        case 'light': return '浅色';
        default: return '跟随系统';
    }
}

export default function App() {
    const [settings, setSettings] = createSignal<AppSettings | null>(null);
    const [recording, setRecording] = createSignal(false);
    const [hotkeyDisplay, setHotkeyDisplay] = createSignal('');

    onMount(async () => {
        await ready();
        const current = await settingsGet();
        setSettings(current);
        setHotkeyDisplay(formatHotkey(current.palette.global_hotkey));
    });

    async function updateTheme(theme: ThemeValue) {
        const current = settings();
        if (!current || current.theme === theme) return;
        const updated = { ...current, theme };
        setSettings(updated);
        await settingsUpdate(updated);
    }

    async function updateHotkey(accelerator: string) {
        const current = settings();
        if (!current) return;
        const updated = { ...current, palette: { ...current.palette, global_hotkey: accelerator } };
        setSettings(updated);
        setHotkeyDisplay(formatHotkey(accelerator));
        await settingsUpdate(updated);
    }

    function handleHotkeyKeyDown(e: KeyboardEvent) {
        if (!recording()) return;
        e.preventDefault();
        e.stopPropagation();

        const parts: string[] = [];
        if (e.metaKey) parts.push('CommandOrControl');
        if (e.ctrlKey && !e.metaKey) parts.push('Control');
        if (e.altKey) parts.push('Alt');
        if (e.shiftKey) parts.push('Shift');

        const key = e.key;
        if (['Meta', 'Control', 'Alt', 'Shift'].includes(key)) return;

        const keyName = key.length === 1 ? key.toUpperCase() : key;
        parts.push(keyName);

        setRecording(false);
        updateHotkey(parts.join('+'));
    }

    return (
        <PluginLayout title="设置">
            <div class="flex flex-col gap-6 max-w-md">
                {/* 主题 */}
                <section>
                    <h2 class="text-sm font-medium text-fg-muted mb-2">主题</h2>
                    <SegmentedControl
                        options={themeOptions}
                        value={(settings()?.theme as ThemeValue) ?? 'system'}
                        onChange={updateTheme}
                    />
                    <p class="text-xs text-fg-muted mt-1">
                        当前：{themeLabel(settings()?.theme ?? 'system')}
                    </p>
                </section>

                {/* 快捷键 */}
                <section>
                    <h2 class="text-sm font-medium text-fg-muted mb-2">全局快捷键</h2>
                    <Input
                        value={recording() ? '按下组合键...' : hotkeyDisplay()}
                        readOnly
                        onFocus={() => setRecording(true)}
                        onBlur={() => setRecording(false)}
                        onKeyDown={handleHotkeyKeyDown}
                        placeholder="点击录制快捷键"
                    />
                    <p class="text-xs text-fg-muted mt-1">
                        点击输入框后按下组合键即可录制
                    </p>
                </section>

                {/* 关于 */}
                <section>
                    <h2 class="text-sm font-medium text-fg-muted mb-2">关于</h2>
                    <p class="text-sm text-fg">litools v0.1.0</p>
                    <p class="text-xs text-fg-muted mt-1">本地效率工具平台</p>
                </section>
            </div>
        </PluginLayout>
    );
}

function formatHotkey(accel: string): string {
    return accel
        .replace(/CommandOrControl/g, navigator.platform.includes('Mac') ? '⌘' : 'Ctrl+')
        .replace(/\+/g, ' + ');
}
