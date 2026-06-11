import { createSignal, createResource, For, Show, onMount } from 'solid-js';
import { ready } from '@litools/plugin-sdk';
import { listPlugins, togglePlugin, installPlugin, uninstallPlugin } from '@litools/plugin-core';
import { PluginLayout, Button, Switch, Badge, Card, ConfirmDialog } from '@litools/plugin-ui';
import { RefreshCw, PackagePlus, Trash2 } from 'lucide-solid';
export default function App() {
    const [view, setView] = createSignal('list');
    const [uninstallTarget, setUninstallTarget] = createSignal(null);
    const [plugins, { refetch }] = createResource(fetchPlugins);
    onMount(async () => {
        await ready();
    });
    async function fetchPlugins() {
        return listPlugins();
    }
    async function handleToggle(pluginId, enabled) {
        try {
            await togglePlugin(pluginId, enabled);
            refetch();
        }
        catch (e) {
            console.error('Toggle failed:', e);
        }
    }
    async function handleInstall() {
        try {
            const { open } = await import('@tauri-apps/plugin-dialog');
            const filePath = await open({
                filters: [{ name: '插件包', extensions: ['zip', 'litools-plugin'] }],
                multiple: false,
            });
            if (filePath && typeof filePath === 'string') {
                await installPlugin(filePath);
                refetch();
            }
        }
        catch (e) {
            console.error('Install failed:', e);
        }
    }
    async function handleUninstallConfirm() {
        const pluginId = uninstallTarget();
        if (!pluginId)
            return;
        try {
            await uninstallPlugin(pluginId);
            setUninstallTarget(null);
            setView('list');
            refetch();
        }
        catch (e) {
            console.error('Uninstall failed:', e);
        }
    }
    const selectedPlugin = () => {
        const v = view();
        if (v === 'list')
            return null;
        return plugins()?.find(p => p.id === v.pluginId) ?? null;
    };
    return (<PluginLayout title="插件管理">
            {/* 工具栏 */}
            <div class="flex items-center gap-2 mb-4">
                <Button variant="primary" size="sm" onClick={handleInstall}>
                    <PackagePlus class="w-4 h-4"/> 安装
                </Button>
                <Button variant="ghost" size="sm" onClick={() => refetch()}>
                    <RefreshCw class="w-4 h-4"/> 刷新
                </Button>
                <Show when={view() !== 'list'}>
                    <Button variant="ghost" size="sm" onClick={() => setView('list')}>
                        ← 返回
                    </Button>
                </Show>
            </div>

            <Show when={view() === 'list'} fallback={<PluginDetailView />}>
                {/* 列表视图 */}
                <Show when={plugins.loading}>
                    <p class="text-sm text-fg-muted">加载中...</p>
                </Show>
                <Show when={plugins.error}>
                    <p class="text-sm text-red-500">加载失败: {plugins.error.message}</p>
                </Show>
                <div class="flex flex-col gap-2">
                    <For each={plugins()}>
                        {(plugin) => (<Card onClick={() => setView({ pluginId: plugin.id })}>
                                <div class="flex items-center gap-3">
                                    <img src={plugin.icon} alt="" class="w-8 h-8 rounded"/>
                                    <div class="flex-1 min-w-0">
                                        <div class="flex items-center gap-2">
                                            <span class="text-sm font-medium truncate">{plugin.name}</span>
                                            <span class="text-xs text-fg-muted">{plugin.version}</span>
                                            <Badge variant={plugin.source} size="sm">{plugin.source}</Badge>
                                            {plugin.trusted && <Badge variant="success" size="sm">受信任</Badge>}
                                        </div>
                                        <p class="text-xs text-fg-muted mt-0.5 line-clamp-1">{plugin.description}</p>
                                    </div>
                                    <Switch checked={plugin.enabled} onChange={(c) => handleToggle(plugin.id, c)}/>
                                    <Button variant="ghost" size="sm" onClick={(e) => { e.stopPropagation(); setUninstallTarget(plugin.id); }}>
                                        <Trash2 class="w-3.5 h-3.5 text-red-500"/>
                                    </Button>
                                </div>
                            </Card>)}
                    </For>
                </div>
            </Show>

            {/* 卸载确认 */}
            <ConfirmDialog open={uninstallTarget() !== null} onClose={() => setUninstallTarget(null)} onConfirm={handleUninstallConfirm} title="卸载插件" message={`确定要卸载 ${plugins()?.find(p => p.id === uninstallTarget())?.name ?? ''} 吗？此操作不可撤销。`} confirmLabel="卸载" variant="danger"/>
        </PluginLayout>);
    function PluginDetailView() {
        const plugin = selectedPlugin();
        if (!plugin)
            return null;
        return (<div class="flex flex-col gap-4">
                {/* 头部 */}
                <div class="flex items-center gap-3">
                    <img src={plugin.icon} alt="" class="w-10 h-10 rounded"/>
                    <div>
                        <h2 class="text-base font-semibold">{plugin.name}</h2>
                        <p class="text-xs text-fg-muted">{plugin.id}</p>
                    </div>
                    <div class="flex-1"/>
                    <Switch checked={plugin.enabled} onChange={(c) => handleToggle(plugin.id, c)} label={plugin.enabled ? '已启用' : '已禁用'}/>
                </div>

                {/* 元信息 */}
                <dl class="grid grid-cols-2 gap-2 text-sm">
                    <dt class="text-fg-muted">版本</dt>
                    <dd>{plugin.version}</dd>
                    <dt class="text-fg-muted">来源</dt>
                    <dd><Badge variant={plugin.source}>{plugin.source}</Badge></dd>
                    <dt class="text-fg-muted">作者</dt>
                    <dd>{plugin.author ?? '—'}</dd>
                    <dt class="text-fg-muted">描述</dt>
                    <dd class="col-span-2">{plugin.description ?? '—'}</dd>
                    <dt class="text-fg-muted">路径</dt>
                    <dd class="text-xs font-mono break-all col-span-2">{plugin.path}</dd>
                </dl>

                {/* 命令列表 */}
                <section>
                    <h3 class="text-sm font-medium text-fg-muted mb-2">命令</h3>
                    <div class="flex flex-col gap-1">
                        <For each={plugin.commands}>
                            {(cmd) => (<div class="flex items-center gap-2 py-1 px-2 rounded bg-bg-muted text-sm">
                                    <span>{cmd.title}</span>
                                    <Badge size="sm">{cmd.mode}</Badge>
                                    <span class="text-xs text-fg-muted">{cmd.id}</span>
                                </div>)}
                        </For>
                    </div>
                </section>

                {/* 权限列表 */}
                <section>
                    <h3 class="text-sm font-medium text-fg-muted mb-2">权限</h3>
                    <div class="flex flex-wrap gap-1">
                        <For each={plugin.permissions}>
                            {(perm) => <Badge size="sm">{perm}</Badge>}
                        </For>
                    </div>
                </section>
            </div>);
    }
}
