import { createResource, For, Show } from 'solid-js';
import { getDiagnostics } from '../../bridge/commands';

export function DiagnosticsPage() {
    const [diagnostics, { refetch }] = createResource(getDiagnostics);

    return (
        <section class="rounded-[20px] border border-border bg-surface p-6 shadow-[var(--shadow-panel)]">
            <div class="flex items-start justify-between gap-4">
                <div>
                    <h1 class="m-0 text-2xl font-semibold">诊断</h1>
                    <p class="m-0 mt-2 text-sm text-muted">查看 litools 的运行状态和本地数据。</p>
                </div>
                <button class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-accent-fg" onClick={() => void refetch()} type="button">
                    刷新
                </button>
            </div>

            <Show when={diagnostics()} fallback={<p class="mt-6 text-sm text-muted">正在加载诊断信息...</p>}>
                {(diagnostics) => (
                    <>
                        <div class="mt-6 grid gap-4 text-sm">
                            <DiagnosticRow label="应用版本" value={diagnostics().app_version} />
                            <DiagnosticRow label="平台" value={diagnostics().platform} />
                            <DiagnosticRow label="应用数据目录" value={diagnostics().app_data_dir} />
                            <DiagnosticRow label="已安装插件" value={String(diagnostics().plugin_count)} />
                            <DiagnosticRow label="已索引命令" value={String(diagnostics().command_count)} />
                            <DiagnosticRow label="最近使用次数" value={String(diagnostics().recent_usage_count)} />
                            <DiagnosticRow label="主题" value={themeLabel(diagnostics().settings.theme)} />
                            <DiagnosticRow label="结果数量上限" value={String(diagnostics().settings.palette.result_limit)} />
                            <DiagnosticRow label="已启用的数据源" value={diagnostics().settings.search.enabled_providers.map(providerLabel).join('，')} />
                            <DiagnosticRow label="窗口行为" value={windowBehaviorSummary(diagnostics().settings)} />
                            <DiagnosticRow label="全局快捷键" value={diagnostics().shortcut.accelerator} />
                            <DiagnosticRow label="快捷键状态" value={diagnostics().shortcut.registered ? '已注册' : (diagnostics().shortcut.error ?? '未注册')} />
                        </div>

                        <div class="mt-8">
                            <h2 class="m-0 text-lg font-semibold">最近使用</h2>
                            <div class="mt-3 grid gap-2 text-sm">
                                <Show when={diagnostics().recent_usage.length} fallback={<p class="m-0 text-muted">暂无使用记录</p>}>
                                    <For each={diagnostics().recent_usage}>
                                        {(event) => (
                                            <div class="grid gap-1 rounded-xl bg-surface-muted px-4 py-3">
                                                <div class="flex items-center justify-between gap-4">
                                                    <span class="font-medium">{event.target_id}</span>
                                                    <span class="text-xs text-muted">{targetTypeLabel(event.target_type)}</span>
                                                </div>
                                                <span class="text-xs text-muted">{event.selected_at}</span>
                                            </div>
                                        )}
                                    </For>
                                </Show>
                            </div>
                        </div>
                    </>
                )}
            </Show>
        </section>
    );
}

function DiagnosticRow(props: { label: string; value: string }) {
    return (
        <div class="grid gap-1 rounded-xl bg-surface-muted px-4 py-3 sm:flex sm:items-center sm:justify-between sm:gap-4">
            <span class="text-muted">{props.label}</span>
            <span class="break-all font-medium">{props.value}</span>
        </div>
    );
}

function providerLabel(provider: string) {
    if (provider === 'commands') {
        return '命令';
    }

    return provider;
}

function targetTypeLabel(targetType: string) {
    if (targetType === 'command') {
        return '命令';
    }

    return targetType;
}

function themeLabel(theme: string) {
    if (theme === 'dark') {
        return '深色';
    }

    if (theme === 'light') {
        return '浅色';
    }

    return '跟随系统';
}

function windowBehaviorSummary(settings: { window: { hide_on_blur: boolean; close_to_tray: boolean; center_on_show: boolean } }) {
    return [
        settings.window.hide_on_blur ? '失焦时隐藏' : '失焦时保持显示',
        settings.window.close_to_tray ? '关闭到托盘' : '关闭即退出',
        settings.window.center_on_show ? '显示时居中' : '保持窗口位置',
    ].join('，');
}
