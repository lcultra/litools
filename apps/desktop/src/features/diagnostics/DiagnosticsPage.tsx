import { createResource, For, onCleanup, onMount, Show } from 'solid-js';
import { getDiagnostics } from '../../bridge/commands';
import { onIndexStatusChanged } from '../../bridge/events';
import { Button } from '../../components/Button';
import { InfoRow } from '../../components/InfoRow';
import { PageHeader } from '../../components/PageHeader';
import { PageState } from '../../components/PageState';
import { providerListLabel, targetTypeLabel } from '../../shared/labels';
import { themeLabel } from '../../shared/theme';
import { iconCacheSummary, indexStatusSummary, reloadSummary, watchedPathsSummary, watcherSummary, windowBehaviorSummary } from './formatters';

export function DiagnosticsPage() {
    const [diagnostics, { refetch }] = createResource(getDiagnostics);

    onMount(() => {
        const unsubscribe = onIndexStatusChanged(() => {
            void refetch();
        });

        onCleanup(() => {
            void unsubscribe.then((dispose) => dispose());
        });
    });

    return (
        <>
            <PageHeader
                action={
                    <Button disabled={diagnostics.loading} onClick={() => void refetch()} variant="primary">
                        {diagnostics.loading ? '正在刷新...' : '刷新'}
                    </Button>
                }
                description="查看 litools 的运行状态和本地数据。"
                title="诊断"
            />

            <Show
                when={!diagnostics.error}
                fallback={
                    <div class="mt-6">
                        <PageState action={{ label: '重试', onClick: () => void refetch() }} description={String(diagnostics.error)} title="诊断信息加载失败" variant="error" />
                    </div>
                }
            >
                <Show
                    when={diagnostics()}
                    fallback={
                        <div class="mt-6">
                            <PageState title="正在加载诊断信息..." variant="loading" />
                        </div>
                    }
                >
                    {(diagnostics) => (
                        <>
                            <div class="mt-6 grid gap-4 text-sm">
                                <InfoRow label="应用版本" value={diagnostics().app_version} />
                                <InfoRow label="平台" value={diagnostics().platform} />
                                <InfoRow label="应用数据目录" value={diagnostics().app_data_dir} />
                                <InfoRow label="已安装插件" value={String(diagnostics().plugin_count)} />
                                <InfoRow label="已索引命令" value={String(diagnostics().command_count)} />
                                <InfoRow label="已索引应用" value={String(diagnostics().app_count)} />
                                <InfoRow label="应用索引状态" value={indexStatusSummary(diagnostics().index_status)} />
                                <InfoRow label="最近索引刷新" value={reloadSummary(diagnostics().last_persisted_index_status ?? diagnostics().index_status.lastSummary)} />
                                <InfoRow label="应用监听" value={watcherSummary(diagnostics().app_watcher)} />
                                <InfoRow label="监听目录" value={watchedPathsSummary(diagnostics().app_watcher.watchedPaths)} />
                                <InfoRow label="图标缓存" value={iconCacheSummary(diagnostics().icon_cache)} />
                                <InfoRow label="最近使用次数" value={String(diagnostics().recent_usage_count)} />
                                <InfoRow label="主题" value={themeLabel(diagnostics().settings.theme)} />
                                <InfoRow label="已启用的数据源" value={providerListLabel(diagnostics().settings.search.enabled_providers)} />
                                <InfoRow label="窗口行为" value={windowBehaviorSummary(diagnostics().settings)} />
                                <InfoRow label="全局快捷键" value={diagnostics().shortcut.accelerator} />
                                <InfoRow label="快捷键状态" value={diagnostics().shortcut.registered ? '已注册' : (diagnostics().shortcut.error ?? '未注册')} />
                            </div>

                            <div class="mt-8">
                                <h2 class="m-0 text-lg font-semibold">最近使用</h2>
                                <div class="mt-3 grid gap-2 text-sm">
                                    <Show when={diagnostics().recent_usage.length} fallback={<PageState title="暂无使用记录" variant="empty" />}>
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
            </Show>
        </>
    );
}
