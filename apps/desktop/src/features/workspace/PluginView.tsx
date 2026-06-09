import { createEffect, createResource, onCleanup, Show } from 'solid-js';
import { getPluginViewDescriptor, hidePluginView, openPluginView } from '../../bridge/commands';
import type { PluginViewState } from '../../bridge/types';
import { PageState } from '../../components/PageState';
import { type AppRoutePath, pluginRouteParts } from '../../views/registry';

export function PluginView(props: { onStateChange?: (state: PluginViewState | null) => void; path: AppRoutePath }) {
    const routeParts = () => pluginRouteParts(props.path);
    const [descriptor] = createResource(routeParts, (parts) => (parts ? getPluginViewDescriptor(parts.pluginId, parts.commandId) : Promise.reject(new Error('无效的插件路径'))));

    let dockedKey: string | null = null;

    createEffect(() => {
        const d = descriptor();
        if (d) {
            props.onStateChange?.({
                pluginId: d.pluginId,
                commandId: d.commandId,
                pluginName: d.pluginName,
                title: d.title,
                lifecycle: 'created',
                placement: 'docked',
                runtimeId: null,
            });
        } else {
            props.onStateChange?.(null);
        }
    });

    createEffect(() => {
        const parts = routeParts();
        const d = descriptor();
        if (!parts || !d) {
            return;
        }

        const key = `${parts.pluginId}:${parts.commandId}`;
        if (dockedKey === key) {
            return;
        }

        dockedKey = key;
        void openPluginView(parts.pluginId, parts.commandId);
    });

    onCleanup(() => {
        const parts = routeParts();
        if (parts) {
            void hidePluginView(parts.pluginId, parts.commandId);
        }
        props.onStateChange?.(null);
    });

    return (
        <div class="flex h-full min-h-0 flex-col bg-white text-fg dark:bg-slate-950">
            <Show
                when={!descriptor.error}
                fallback={
                    <div class="p-4">
                        <PageState description={String(descriptor.error)} title="插件加载失败" variant="error" />
                    </div>
                }
            >
                <Show
                    when={descriptor()}
                    fallback={
                        <div class="p-4">
                            <PageState title="正在加载插件..." variant="loading" />
                        </div>
                    }
                >
                    <div class="grid min-h-0 flex-1 place-items-center text-muted text-sm">插件页面由独立原生视图承载</div>
                </Show>
            </Show>
        </div>
    );
}
