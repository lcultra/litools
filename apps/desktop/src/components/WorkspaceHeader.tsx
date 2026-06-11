import { LogicalPosition } from '@tauri-apps/api/dpi';
import { Menu } from '@tauri-apps/api/menu';
import { resolveResource } from '@tauri-apps/api/path';
import { EllipsisVertical, X } from 'lucide-solid';
import { createSignal, For, Show } from 'solid-js';
import { detachPluginView, openPluginDevtools, startWindowDragging } from '../bridge/commands';
import type { PluginViewState } from '../bridge/types';

const MENU_OFFSET_Y = 8;

const detachIcon = () => resolveResource('resources/menu/split_window.png');
const closeIcon = () => resolveResource('resources/menu/close.png');
const debugIcon = () => resolveResource('resources/menu/debug.png');

type WorkspaceHeaderProps = {
    commandId?: string;
    icon?: string;
    isDetached?: boolean;
    onClose: () => void;
    pluginId?: string;
    pluginView: PluginViewState | null;
    runtimeId: string | null;
};

export function WorkspaceHeader(props: WorkspaceHeaderProps) {
    const [menuError, setMenuError] = createSignal<string | null>(null);
    const breadcrumbs = () => (props.pluginView ? [props.pluginView.pluginName, props.pluginView.title] : ['插件']);

    function handleDragPointerDown(event: PointerEvent) {
        if (event.button !== 0) {
            return;
        }

        void startWindowDragging();
    }

    async function showMenu(event: MouseEvent) {
        const rect = (event.currentTarget as HTMLButtonElement).getBoundingClientRect();
        setMenuError(null);

        try {
            const menu = await Menu.new({
                items: props.isDetached ? await detachedMenuItems() : await dockedMenuItems(),
            });

            await menu.popup(new LogicalPosition(rect.left, rect.bottom + MENU_OFFSET_Y));
        } catch (error) {
            setMenuError(`打开菜单失败：${String(error)}`);
        }
    }

    async function dockedMenuItems() {
        const detach = {
            id: 'detach',
            text: '分离为独立窗口',
            icon: await detachIcon(),
            accelerator: 'CmdOrCtrl+D',
            action: () => {
                if (props.pluginId && props.commandId) {
                    void detachPluginView(props.pluginId, props.commandId);
                }
            },
        };
        const close = {
            id: 'close',
            text: '结束运行',
            icon: await closeIcon(),
            action: props.onClose,
        };
        if (props.pluginView?.dev) {
            const devtools = {
                id: 'devtools',
                text: '开发者工具',
                icon: await debugIcon(),
                action: () => {
                    if (props.runtimeId) {
                        void openPluginDevtools(props.runtimeId);
                    }
                },
            };
            return [detach, devtools, close];
        }
        return [detach, close];
    }

    async function detachedMenuItems() {
        const close = {
            id: 'close',
            text: '结束运行',
            icon: await closeIcon(),
            action: props.onClose,
        };
        if (props.pluginView?.dev) {
            const devtools = {
                id: 'devtools',
                text: '开发者工具',
                icon: await debugIcon(),
                action: () => {
                    if (props.runtimeId) {
                        void openPluginDevtools(props.runtimeId);
                    }
                },
            };
            return [devtools, close];
        }
        return [close];
    }

    return (
        <header class="flex h-17 shrink-0 items-center gap-2 border-border border-b px-3 box-border">
            <div class="flex items-center overflow-hidden rounded-full border border-border bg-surface-muted text-sm">
                <Show when={props.icon}>
                    <img alt="" class="ml-2 size-6 shrink-0 object-contain" draggable={false} onPointerDown={handleDragPointerDown} src={props.icon} />
                </Show>
                <div class="flex items-center gap-2 py-1.5 pr-2 select-none" classList={{ 'pl-3': !props.icon, 'pl-1.5': !!props.icon }} onPointerDown={handleDragPointerDown}>
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
                    data-interactive
                    onClick={props.onClose}
                    type="button"
                >
                    <X size={16} strokeWidth={2} />
                </button>
            </div>
            <div aria-hidden="true" class="min-w-0 flex-1 self-stretch cursor-grab active:cursor-grabbing" onPointerDown={handleDragPointerDown} />
            <button
                aria-label="打开菜单"
                class="grid size-8 cursor-pointer place-items-center rounded-full border border-border text-muted outline-none transition-colors hover:bg-surface/80 hover:text-fg focus-visible:ring-2 focus-visible:ring-accent/30 focus-visible:outline-none"
                data-interactive
                onClick={showMenu}
                title={menuError() ?? '更多操作'}
                type="button"
            >
                <EllipsisVertical size={16} strokeWidth={2} />
            </button>
        </header>
    );
}
