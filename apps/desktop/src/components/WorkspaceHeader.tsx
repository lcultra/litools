import { useLocation } from '@solidjs/router';
import { LogicalPosition } from '@tauri-apps/api/dpi';
import { Menu } from '@tauri-apps/api/menu';
import { Ellipsis, X } from 'lucide-solid';
import { createSignal, For, Show } from 'solid-js';
import { detachPluginView, detachRoute, startWindowDragging } from '../bridge/commands';
import type { PluginViewState } from '../bridge/types';
import { canDetachRoute, pluginRouteParts } from '../views/registry';

const MENU_OFFSET_Y = 8;

type WorkspaceHeaderProps = {
    isDetached?: boolean;
    onClose: () => void;
    ownerReady?: boolean;
    pluginView: PluginViewState | null;
};

export function WorkspaceHeader(props: WorkspaceHeaderProps) {
    const location = useLocation();
    const [menuError, setMenuError] = createSignal<string | null>(null);
    const currentPath = () => location.pathname;
    const breadcrumbs = () => (props.pluginView ? [props.pluginView.pluginName, props.pluginView.title] : ['插件']);
    const canDetach = () => Boolean(props.ownerReady) && !props.isDetached && canDetachRoute(currentPath());

    function handleDragPointerDown(event: PointerEvent) {
        if (event.button !== 0) {
            return;
        }

        void startWindowDragging();
    }

    async function showMenu(event: MouseEvent) {
        const path = currentPath();

        if (!canDetachRoute(path)) {
            return;
        }

        const rect = (event.currentTarget as HTMLButtonElement).getBoundingClientRect();
        setMenuError(null);

        try {
            const menu = await Menu.new({
                items: [
                    {
                        action: () => {
                            const parts = pluginRouteParts(path);
                            if (parts) {
                                void detachPluginView(parts.pluginId, parts.commandId);
                                return;
                            }
                            void detachRoute(path);
                        },
                        id: 'detach',
                        text: '分离为独立窗口',
                    },
                ],
            });

            await menu.popup(new LogicalPosition(rect.left, rect.bottom + MENU_OFFSET_Y));
        } catch (error) {
            setMenuError(`打开菜单失败：${String(error)}`);
        }
    }

    return (
        <header class="flex h-17 shrink-0 items-center gap-2 border-border border-b px-3">
            <div class="flex items-center overflow-hidden rounded-full border border-border bg-surface-muted text-sm">
                <div class="flex items-center gap-2 py-1.5 pl-3 pr-2" onPointerDown={handleDragPointerDown}>
                    <For each={breadcrumbs()}>
                        {(breadcrumb, index) => (
                            <>
                                <Show when={index() > 0}>
                                    <span class="text-muted">/</span>
                                </Show>
                                <span class={index() === 0 ? 'font-semibold text-fg' : 'text-muted'}>{breadcrumb}</span>
                            </>
                        )}
                    </For>
                </div>
                <button
                    aria-label="关闭插件视图"
                    class="grid size-8 cursor-pointer place-items-center border-border border-l text-muted outline-none transition-colors hover:bg-danger/10 hover:text-danger focus-visible:ring-2 focus-visible:ring-accent/30 focus-visible:outline-none"
                    onClick={props.onClose}
                    type="button"
                >
                    <X size={16} strokeWidth={2} />
                </button>
            </div>
            <div aria-hidden="true" class="min-w-0 flex-1 self-stretch cursor-grab active:cursor-grabbing" onPointerDown={handleDragPointerDown} />
            <Show when={canDetach()}>
                <button
                    aria-label="打开菜单"
                    class="grid size-8 cursor-pointer place-items-center rounded-full border border-border text-muted outline-none transition-colors hover:bg-surface/80 hover:text-fg focus-visible:ring-2 focus-visible:ring-accent/30 focus-visible:outline-none"
                    onClick={showMenu}
                    title={menuError() ?? '更多操作'}
                    type="button"
                >
                    <Ellipsis size={16} strokeWidth={2} />
                </button>
            </Show>
        </header>
    );
}
