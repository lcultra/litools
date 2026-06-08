import { createEffect, createResource, onCleanup, Show } from 'solid-js';
import { getPluginRuntimeDescriptor } from '../../bridge/commands';
import { PageState } from '../../components/PageState';
import { type AppRoutePath, pluginRuntimeRouteParts } from '../../views/registry';

export function PluginRuntimePage(props: { onBreadcrumbsChange?: (breadcrumbs: string[] | null) => void; path: AppRoutePath }) {
    const routeParts = () => pluginRuntimeRouteParts(props.path);
    const [descriptor] = createResource(routeParts, (parts) =>
        parts ? getPluginRuntimeDescriptor(parts.pluginId, parts.commandId) : Promise.reject(new Error('无效的插件运行时路径')),
    );

    createEffect(() => {
        const runtime = descriptor();
        props.onBreadcrumbsChange?.(runtime ? [runtime.pluginName, runtime.title] : null);
    });

    onCleanup(() => props.onBreadcrumbsChange?.(null));

    return (
        <div class="flex h-full min-h-0 flex-col text-fg">
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
                    {(runtime) => <iframe class="min-h-0 flex-1 border-0 bg-white" src={runtime().entryUrl} title={runtime().title} />}
                </Show>
            </Show>
        </div>
    );
}
