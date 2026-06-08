import { createResource, Show } from 'solid-js';
import { closePluginRuntimeById, getPluginRuntimeInfo, startWindowDragging } from '../../bridge/commands';
import { PageState } from '../../components/PageState';
import { type AppRoutePath, pluginRuntimeHeaderRouteParts } from '../../views/registry';

export function PluginRuntimeHeaderPage(props: { path: AppRoutePath }) {
    const routeParts = () => pluginRuntimeHeaderRouteParts(props.path);
    const [runtime] = createResource(routeParts, (parts) => (parts ? getPluginRuntimeInfo(parts.runtimeId) : Promise.reject(new Error('无效的插件标题栏路径'))));

    function handleDragPointerDown(event: PointerEvent) {
        if (event.button !== 0) {
            return;
        }

        void startWindowDragging();
    }

    return (
        <header class="flex h-screen items-center gap-2 border-border border-b bg-bg px-3 text-fg">
            <Show when={!runtime.error} fallback={<PageState description={String(runtime.error)} title="标题栏加载失败" variant="error" />}>
                <Show when={runtime()} fallback={<PageState title="正在加载标题栏..." variant="loading" />}>
                    {(info) => (
                        <>
                            <div class="min-w-0 flex-1 cursor-grab active:cursor-grabbing" onPointerDown={handleDragPointerDown}>
                                <div class="truncate font-semibold text-sm">{info().pluginName}</div>
                                <div class="truncate text-muted text-xs">{info().title}</div>
                            </div>
                            <button
                                class="rounded-full border border-border px-3 py-1 text-muted text-xs transition-colors hover:bg-surface-muted hover:text-fg"
                                disabled
                                type="button"
                            >
                                操作
                            </button>
                            <button
                                class="rounded-full border border-border px-3 py-1 text-danger text-xs transition-colors hover:bg-danger/10"
                                onClick={() => void closePluginRuntimeById(info().runtimeId)}
                                type="button"
                            >
                                关闭
                            </button>
                        </>
                    )}
                </Show>
            </Show>
        </header>
    );
}
