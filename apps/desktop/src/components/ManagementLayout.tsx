import type { JSX } from 'solid-js';
import { Show } from 'solid-js';
import { ManagementHeader } from './ManagementHeader';
import { ManagementNav } from './ManagementNav';
import { WindowFrame } from './WindowFrame';

type ManagementLayoutMode = 'center' | 'standalone';

type ManagementLayoutProps = {
    children: JSX.Element;
    isDetached?: boolean;
    mode?: ManagementLayoutMode;
    ownerReady?: boolean;
    onOpenLauncher: () => void;
};

export function ManagementLayout(props: ManagementLayoutProps) {
    const mode = () => props.mode ?? 'center';

    return (
        <WindowFrame class="flex h-[calc(100vh-2px)] flex-col">
            <ManagementHeader isDetached={props.isDetached} ownerReady={props.ownerReady} onClose={props.onOpenLauncher} />
            <div class="flex min-h-0 flex-1">
                <Show when={mode() === 'center'}>
                    <ManagementNav />
                </Show>
                <section class="min-w-0 flex-1 overflow-y-auto p-6">{props.children}</section>
            </div>
        </WindowFrame>
    );
}
