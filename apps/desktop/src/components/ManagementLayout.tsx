import type { JSX } from 'solid-js';
import { Show } from 'solid-js';
import { ManagementNav } from './ManagementNav';

type ManagementLayoutMode = 'center' | 'standalone';

type ManagementLayoutProps = {
    children: JSX.Element;
    mode?: ManagementLayoutMode;
    onOpenLauncher: () => void;
};

export function ManagementLayout(props: ManagementLayoutProps) {
    const mode = () => props.mode ?? 'center';

    return (
        <div class="h-screen overflow-hidden rounded-[20px] bg-surface shadow-[inset_0_0_0_1px_var(--border)]">
            <div class="flex h-full min-h-0">
                <Show when={mode() === 'center'}>
                    <ManagementNav onOpenLauncher={props.onOpenLauncher} />
                </Show>
                <section class="min-w-0 flex-1 overflow-y-auto p-6">{props.children}</section>
            </div>
        </div>
    );
}
