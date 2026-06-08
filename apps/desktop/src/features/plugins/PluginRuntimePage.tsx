import { createEffect, createResource, onCleanup, Show } from 'solid-js';
import { dockPluginRuntime, getPluginRuntimeDescriptor, hideDockedPluginRuntime } from '../../bridge/commands';
import { PageState } from '../../components/PageState';
import { type AppRoutePath, pluginRuntimeRouteParts } from '../../views/registry';

export function PluginRuntimePage(props: { onBreadcrumbsChange?: (breadcrumbs: string[] | null) => void; path: AppRoutePath }) {
    const routeParts = () => pluginRuntimeRouteParts(props.path);
    const [descriptor] = createResource(routeParts, (parts) =>
        parts ? getPluginRuntimeDescriptor(parts.pluginId, parts.commandId) : Promise.reject(new Error('无效的插件运行时路径')),
    );

    let dockedKey: string | null = null;

    createEffect(() => {
        const runtime = descriptor();
        props.onBreadcrumbsChange?.(runtime ? [runtime.pluginName, runtime.title] : null);
    });

    createEffect(() => {
        const parts = routeParts();
        const runtime = descriptor();
        if (!parts || !runtime) {
            return;
        }

        const key = `${parts.pluginId}:${parts.commandId}`;
        if (dockedKey === key) {
            return;
        }

        dockedKey = key;
        void dockPluginRuntime(parts.pluginId, parts.commandId);
    });

    onCleanup(() => {
        const parts = routeParts();
        if (parts) {
            void hideDockedPluginRuntime(parts.pluginId, parts.commandId);
        }
        props.onBreadcrumbsChange?.(null);
    });

    return (
        <div class="flex h-full min-h-0 flex-col bg-white text-fg dark:bg-slate-950">
            <Show
                when={!descriptor.error}
                fallback={
                    <div class="p-4">
                        <PageState description={String(descriptor.error)} title="插件运行时加载失败" variant="error" />
                    </div>
                }
            >
                <Show
                    when={descriptor()}
                    fallback={
                        <div class="p-4">
                            <PageState title="正在加载插件运行时..." variant="loading" />
                        </div>
                    }
                >
                    {() => <div class="grid min-h-0 flex-1 place-items-center text-muted text-sm">插件页面由独立原生视图承载</div>}
                </Show>
            </Show>
        </div>
    );
}
