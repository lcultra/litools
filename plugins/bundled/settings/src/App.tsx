import { createSignal, onMount } from 'solid-js';
import { runtime, settings } from '@litools/sdk';
import type { ThemeValue, AppSettings } from '@litools/sdk';
import { PluginLayout, SegmentedControl, Input } from '@litools/ui';

const THEME_ITEMS = [
    { value: 'system', label: '跟随系统' },
    { value: 'light', label: '浅色' },
    { value: 'dark', label: '深色' },
] as const;

function formatHotkey(accel: string): string {
    return accel
        .replace(/CommandOrControl/g, navigator.platform.includes('Mac') ? '⌘' : 'Ctrl+')
        .replace(/\+/g, ' + ');
}

export default function App() {
    const [appSettings, setAppSettings] = createSignal<AppSettings | null>(null);
    const [recording, setRecording] = createSignal(false);
    const [hotkeyDisplay, setHotkeyDisplay] = createSignal('');
    let hotkeyInput!: HTMLInputElement;

    onMount(async () => {
        await runtime.ready().catch(() => {});
        const current = await settings.get().catch(() => null);
        if (current) {
            setAppSettings(current);
            setHotkeyDisplay(formatHotkey(current.palette.global_hotkey));
        }
    });

    async function updateTheme(theme: string) {
        const current = appSettings();
        if (current?.theme === theme) return;
        const base = current ?? ({} as AppSettings);
        const updated = { ...base, theme: theme as ThemeValue };
        setAppSettings(updated);
        await settings.update(updated).catch(() => {});
    }

    async function updateHotkey(accelerator: string) {
        const current = appSettings();
        if (!current) return;
        const updated = { ...current, palette: { ...current.palette, global_hotkey: accelerator } };
        setAppSettings(updated);
        setHotkeyDisplay(formatHotkey(accelerator));
        await settings.update(updated);
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
        parts.push(key.length === 1 ? key.toUpperCase() : key);
        setRecording(false);
        updateHotkey(parts.join('+'));
    }

    return (
        <PluginLayout title="设置">
            <div class="flex flex-col gap-6">
                {/* 主题 */}
                <section>
                    <h2 class="text-sm font-medium text-text-muted mb-2">主题</h2>
                    <SegmentedControl
                        items={THEME_ITEMS}
                        value={appSettings()?.theme ?? 'system'}
                        onChange={updateTheme}
                    />
                </section>

                {/* 快捷键 */}
                <section>
                    <h2 class="text-sm font-medium text-text-muted mb-2">全局快捷键</h2>
                    <Input
                        value={recording() ? '按下组合键...' : hotkeyDisplay()}
                        placeholder="点击录制快捷键"
                        readOnly
                        onFocus={() => setRecording(true)}
                        onBlur={() => setRecording(false)}
                        onKeyDown={handleHotkeyKeyDown}
                        ref={(el: HTMLInputElement) => { hotkeyInput = el; }}
                    />
                    <p class="text-xs text-text-muted mt-1">点击输入框后按下组合键即可录制</p>
                </section>

                {/* 关于 */}
                <section>
                    <h2 class="text-sm font-medium text-text-muted mb-2">关于</h2>
                    <p class="text-sm text-text">litools v0.1.0</p>
                    <p class="text-xs text-text-muted mt-1">本地效率工具平台</p>
                </section>
            </div>
        </PluginLayout>
    );
}
