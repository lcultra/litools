import type { JSX } from 'solid-js';
import type { PluginViewState } from '../bridge/types';
import { WindowFrame } from './WindowFrame';
import { WorkspaceHeader } from './WorkspaceHeader';

type WorkspaceViewProps = {
    children: JSX.Element;
    isDetached?: boolean;
    ownerReady?: boolean;
    pluginView: PluginViewState | null;
    onClose: () => void;
};

export function WorkspaceView(props: WorkspaceViewProps) {
    return (
        <WindowFrame class="flex h-[calc(100vh-2px)] flex-col">
            <WorkspaceHeader isDetached={props.isDetached} onClose={props.onClose} ownerReady={props.ownerReady} pluginView={props.pluginView} />
            <div class="flex min-h-0 flex-1">
                <section class="min-w-0 flex-1">{props.children}</section>
            </div>
        </WindowFrame>
    );
}
