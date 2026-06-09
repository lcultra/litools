import { useNavigate, useParams } from '@solidjs/router';
import { createEffect, createResource, createSignal, onCleanup, Show } from 'solid-js';
import { useShell } from '../../App';
import { closePluginView, getPluginViewDescriptor, openPluginView } from '../../bridge/commands';
import type { PluginViewState } from '../../bridge/types';
import { PageState } from '../../components/PageState';
import { WindowFrame } from '../../components/WindowFrame';
import { WorkspaceHeader } from '../../components/WorkspaceHeader';

export function WorkspacePage() {
    const params = useParams<{ pluginId: string; commandId: string }>();
    const navigate = useNavigate();
    const { isDetachedWindow, hostWindowLabel } = useShell();

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
                runtimeId: null,
            });
        }
    });

    // Open plugin webview when mounted, close on unmount.
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
        void closePluginView(params.pluginId, params.commandId);
    });

    function handleClose() {
        void closePluginView(params.pluginId, params.commandId);
        navigate('/');
    }

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
                    <WorkspaceHeader isDetached={isDetachedWindow()} onClose={handleClose} ownerReady={Boolean(hostWindowLabel())} pluginView={pluginView()} />
                    <div class="flex min-h-0 flex-1">
                        <section class="min-w-0 flex-1" />
                    </div>
                </Show>
            </Show>
        </WindowFrame>
    );
}
