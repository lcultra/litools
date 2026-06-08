import type { JSX } from 'solid-js';
import { createResource, For, Show } from 'solid-js';
import { listPlugins } from '../../bridge/commands';
import type { PluginCommandMode, PluginSource, PluginSummary } from '../../bridge/types';
import { Button } from '../../components/Button';
import { PageHeader } from '../../components/PageHeader';
import { PageState } from '../../components/PageState';

export function PluginManagerPage() {
    const [plugins, { refetch }] = createResource(listPlugins);

    return (
        <>
            <PageHeader
                action={
                    <div class="flex gap-2">
                        <Button disabled variant="secondary">
                            插件市场即将支持
                        </Button>
                        <Button disabled variant="secondary">
                            本地安装即将支持
                        </Button>
                        <Button disabled={plugins.loading} onClick={() => void refetch()} variant="primary">
                            {plugins.loading ? '正在刷新...' : '刷新'}
                        </Button>
                    </div>
                }
                description="管理插件生命周期、配置和扩展能力。"
                title="插件中心"
            />

            <Show
                when={!plugins.error}
                fallback={
                    <div class="mt-6">
                        <PageState action={{ label: '重试', onClick: () => void refetch() }} description={String(plugins.error)} title="插件列表加载失败" variant="error" />
                    </div>
                }
            >
                <Show
                    when={plugins()}
                    fallback={
                        <div class="mt-6">
                            <PageState title="正在加载插件列表..." variant="loading" />
                        </div>
                    }
                >
                    {(pluginList) => (
                        <div class="mt-6 grid gap-4">
                            <Show
                                when={pluginList().length > 0}
                                fallback={
                                    <PageState
                                        description="未发现内置插件资源。用户插件仍可放入应用数据目录的 plugins 文件夹后重启发现。"
                                        title="暂无可管理的插件"
                                        variant="empty"
                                    />
                                }
                            >
                                <For each={pluginList()}>{(plugin) => <PluginCard plugin={plugin} />}</For>
                            </Show>
                        </div>
                    )}
                </Show>
            </Show>
        </>
    );
}

function PluginCard(props: { plugin: PluginSummary }) {
    return (
        <article class="rounded-xl bg-surface-muted px-4 py-4 text-sm shadow-sm">
            <div class="flex items-start justify-between gap-4">
                <div class="min-w-0">
                    <div class="flex flex-wrap items-center gap-2">
                        <h2 class="m-0 text-base font-semibold text-fg">{props.plugin.name}</h2>
                        <Badge tone="neutral">v{props.plugin.version}</Badge>
                        <Badge tone={props.plugin.source === 'bundled' ? 'success' : 'neutral'}>{pluginSourceLabel(props.plugin.source)}</Badge>
                        <Badge tone={props.plugin.enabled ? 'success' : 'muted'}>{props.plugin.enabled ? '已启用' : '已停用'}</Badge>
                        <Badge tone={props.plugin.trusted ? 'success' : 'warning'}>
                            {props.plugin.source === 'bundled' && props.plugin.trusted ? '内置信任' : props.plugin.trusted ? '已信任' : '未信任'}
                        </Badge>
                    </div>
                    <p class="m-0 mt-1 text-xs text-muted">{props.plugin.id}</p>
                    <Show when={props.plugin.description}>{(description) => <p class="m-0 mt-3 text-muted">{description()}</p>}</Show>
                    <Show when={props.plugin.author}>{(author) => <p class="m-0 mt-2 text-xs text-muted">作者：{author()}</p>}</Show>
                </div>
            </div>

            <div class="mt-4 grid gap-3">
                <section>
                    <h3 class="m-0 text-xs font-semibold uppercase tracking-wide text-muted">命令</h3>
                    <Show when={props.plugin.commands.length > 0} fallback={<p class="m-0 mt-2 text-xs text-muted">未声明命令</p>}>
                        <div class="mt-2 grid gap-2">
                            <For each={props.plugin.commands}>
                                {(command) => (
                                    <div class="rounded-lg bg-surface px-3 py-2">
                                        <div class="flex items-center justify-between gap-3">
                                            <span class="font-medium text-fg">{command.title}</span>
                                            <Badge tone="neutral">{commandModeLabel(command.mode)}</Badge>
                                        </div>
                                        <p class="m-0 mt-1 text-xs text-muted">{command.subtitle ?? command.id}</p>
                                    </div>
                                )}
                            </For>
                        </div>
                    </Show>
                </section>

                <section>
                    <h3 class="m-0 text-xs font-semibold uppercase tracking-wide text-muted">权限</h3>
                    <p class="m-0 mt-2 text-xs text-muted">{props.plugin.permissions.length ? props.plugin.permissions.join('、') : '未声明权限'}</p>
                </section>

                <section>
                    <h3 class="m-0 text-xs font-semibold uppercase tracking-wide text-muted">来源路径</h3>
                    <p class="m-0 mt-2 break-all text-xs text-muted">{props.plugin.path}</p>
                </section>
            </div>
        </article>
    );
}

function Badge(props: { children: JSX.Element; tone: 'muted' | 'neutral' | 'success' | 'warning' }) {
    const toneClass = () => {
        switch (props.tone) {
            case 'success':
                return 'bg-success/10 text-success';
            case 'warning':
                return 'bg-warning/10 text-warning';
            case 'neutral':
                return 'bg-surface text-muted';
            case 'muted':
                return 'bg-surface text-muted';
        }
    };

    return <span class={['rounded-full px-2 py-0.5 text-xs font-medium', toneClass()].join(' ')}>{props.children}</span>;
}

function pluginSourceLabel(source: PluginSource) {
    switch (source) {
        case 'bundled':
            return '内置';
        case 'user':
            return '用户安装';
    }
}

function commandModeLabel(mode: PluginCommandMode) {
    switch (mode) {
        case 'instant':
            return '即时命令';
        case 'view':
            return '视图';
        case 'searchProvider':
            return '搜索源';
    }
}
