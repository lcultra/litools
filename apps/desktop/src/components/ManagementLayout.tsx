import type { JSX } from 'solid-js';
import { ManagementHeader } from './ManagementHeader';
import { WindowFrame } from './WindowFrame';

type ManagementLayoutProps = {
    breadcrumbs?: string[];
    children: JSX.Element;
    isDetached?: boolean;
    ownerReady?: boolean;
    onOpenLauncher: () => void;
};

export function ManagementLayout(props: ManagementLayoutProps) {
    return (
        <WindowFrame class="flex h-[calc(100vh-2px)] flex-col">
            <ManagementHeader breadcrumbs={props.breadcrumbs} isDetached={props.isDetached} ownerReady={props.ownerReady} onClose={props.onOpenLauncher} />
            <div class="flex min-h-0 flex-1">
                <section class="min-w-0 flex-1 overflow-y-auto">{props.children}</section>
            </div>
        </WindowFrame>
    );
}
