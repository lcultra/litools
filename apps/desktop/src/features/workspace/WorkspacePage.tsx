import { useLocation, useNavigate, useParams } from '@solidjs/router';
import { createEffect, createResource, createSignal, onCleanup, Show } from 'solid-js';
import { closePluginView, closePluginViewById, detachPluginView, getPluginViewDescriptor, openPluginView } from '../../bridge/commands';
import type { PluginViewState } from '../../bridge/types';
import { PageState } from '../../components/PageState';
import { WindowFrame } from '../../components/WindowFrame';
import { WorkspaceHeader } from '../../components/WorkspaceHeader';
import { useGlobalKey } from '../../shared/keyboard';
import { hostWindowLabel, isDetachedWindow } from '../../shared/store';

export function WorkspacePage() {
    const params = useParams<{ pluginId: string; commandId: string }>();
    const location = useLocation<{ runtimeId?: string }>();
    const navigate = useNavigate();
    const runtimeId = () => (location.state as Record<string, unknown> | null)?.runtimeId as string | undefined;

    const [descriptor] = createResource(
        () => ({ pluginId: params.pluginId, commandId: params.commandId }),
        ({ pluginId, commandId }) => getPluginViewDescriptor(pluginId, commandId),
    );

    const [pluginView, setPluginView] = createSignal<PluginViewState | null>(null);

    createEffect(() => {
        const desc = descriptor();
        if (desc) {
            setPluginView({
                pluginId: desc.pluginId,
                commandId: desc.commandId,
                pluginName: desc.pluginName,
                title: desc.title,
                lifecycle: 'created',
                placement: isDetachedWindow() ? 'detached' : 'docked',
                runtimeId: runtimeId() ?? null,
                dev: desc.dev,
            });
        }
    });

    // 创建/查找运行时。Singleton 时若已由 LauncherPage 创建则为无害的 EnsureVisible。
    let openedKey: string | null = null;
    createEffect(() => {
        const { pluginId, commandId } = params;
        if (!pluginId || !commandId) return;

        const key = `${pluginId}:${commandId}`;
        if (openedKey === key) return;
        openedKey = key;

        void openPluginView(pluginId, commandId);
    });

    onCleanup(() => {
        const rid = runtimeId();
        if (rid) {
            void closePluginViewById(rid);
        } else {
            void closePluginView(params.pluginId, params.commandId);
        }
    });

    function handleClose() {
        const rid = runtimeId();
        if (rid) {
            void closePluginViewById(rid);
        } else {
            void closePluginView(params.pluginId, params.commandId);
        }
        // 分离态：后端 close_runtime 已 destroy 窗口，无需额外操作
        // 停靠态：后端 open_view_route 展示窗口，前端 navigate 驱动 HashRouter
        if (!isDetachedWindow()) {
            navigate('/');
        }
    }

    function handleDetach() {
        if (!isDetachedWindow() && params.pluginId && params.commandId) {
            void detachPluginView(params.pluginId, params.commandId);
        }
    }

    useGlobalKey('Escape', handleClose, { prevent: true });
    useGlobalKey('d', handleDetach, { prevent: true, modifier: 'meta' });

    return (
        <WindowFrame class="flex h-[calc(100vh-2px)] flex-col">
            <Show
                when={!descriptor.error}
                fallback={
                    <div class="p-4">
                        <PageState description={String(descriptor.error)} title="插件加载失败" variant="error" />
                    </div>
                }
            >
                <Show when={descriptor()} fallback={<PageState title="正在加载插件..." variant="loading" />}>
                    <WorkspaceHeader
                        icon={descriptor()?.icon}
                        isDetached={isDetachedWindow()}
                        onClose={handleClose}
                        ownerReady={Boolean(hostWindowLabel())}
                        pluginId={params.pluginId}
                        commandId={params.commandId}
                        pluginView={pluginView()}
                        runtimeId={runtimeId() ?? null}
                    />
                    <div class="flex min-h-0 flex-1">
                        <section class="min-w-0 flex-1" />
                    </div>
                </Show>
            </Show>
        </WindowFrame>
    );
}
