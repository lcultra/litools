import type { JSX } from 'solid-js';
import { RuntimeHeader } from './RuntimeHeader';
import { WindowFrame } from './WindowFrame';

type RuntimeLayoutProps = {
    breadcrumbs?: string[];
    children: JSX.Element;
    isDetached?: boolean;
    ownerReady?: boolean;
    onOpenLauncher: () => void;
};

export function RuntimeLayout(props: RuntimeLayoutProps) {
    return (
        <WindowFrame class="flex h-[calc(100vh-2px)] flex-col">
            <RuntimeHeader breadcrumbs={props.breadcrumbs} isDetached={props.isDetached} ownerReady={props.ownerReady} onClose={props.onOpenLauncher} />
            <div class="flex min-h-0 flex-1">
                <section class="min-w-0 flex-1 overflow-y-auto">{props.children}</section>
            </div>
        </WindowFrame>
    );
}
