import { createResource, Show } from 'solid-js';
import { closePluginViewById, getPluginViewInfo } from '../../bridge/commands';
import type { PluginViewState } from '../../bridge/types';
import { PageState } from '../../components/PageState';
import { WindowFrame } from '../../components/WindowFrame';
import { WorkspaceHeader } from '../../components/WorkspaceHeader';
import { type AppRoutePath, titlebarRouteParts } from '../../views/registry';

export function TitlebarPage(props: { path: AppRoutePath }) {
    const routeParts = () => titlebarRouteParts(props.path);
    const [viewInfo] = createResource(routeParts, (parts) => (parts ? getPluginViewInfo(parts.runtimeId) : Promise.reject(new Error('无效的标题栏路径'))));

    const pluginView = (): PluginViewState | null => {
        const info = viewInfo();
        return info
            ? {
                  pluginId: info.pluginId,
                  commandId: info.commandId,
                  pluginName: info.pluginName,
                  title: info.title,
                  lifecycle: info.lifecycle,
                  placement: info.placement,
                  runtimeId: info.runtimeId,
              }
            : null;
    };

    return (
        <WindowFrame class="flex h-[calc(100vh-2px)] flex-col">
            <Show when={viewInfo()} fallback={<PageState title="正在加载标题栏..." variant="loading" />}>
                {(info) => <WorkspaceHeader isDetached onClose={() => void closePluginViewById(info().runtimeId)} pluginView={pluginView()} />}
            </Show>
        </WindowFrame>
    );
}
