import { LogicalPosition } from '@tauri-apps/api/dpi';
import { Menu } from '@tauri-apps/api/menu';
import { resolveResource } from '@tauri-apps/api/path';
import { revealInFileManager } from '../../bridge/commands';
import type { VisibleLauncherItem } from './LauncherPage';

export type ResultContextMenuActions = {
    onTogglePin: (item: VisibleLauncherItem['item']) => void;
    onError: (message: string) => void;
};

const pinIcon = () => resolveResource('resources/menu/pin-fill.png');
const removeIcon = () => resolveResource('resources/menu/pin.png');
const folderIcon = () => resolveResource('resources/menu/folder.png');

export async function showResultContextMenu(renderItem: VisibleLauncherItem, position: { x: number; y: number }, actions: ResultContextMenuActions) {
    const isApp = renderItem.result.provider === 'apps';
    const isPinned = renderItem.item.isPinned;

    try {
        const menu = await Menu.new({
            items: [
                {
                    id: isPinned ? 'unpin-from-search-bar' : 'pin-to-search-bar',
                    text: isPinned ? '从搜索栏取消固定' : '固定到搜索栏',
                    icon: await (isPinned ? removeIcon() : pinIcon()),
                    action: () => actions.onTogglePin(renderItem.item),
                },
                ...(isApp
                    ? [
                          {
                              id: 'reveal-in-file-manager' as const,
                              text: '打开文件位置' as const,
                              icon: await folderIcon(),
                              action: () => {
                                  void revealInFileManager(renderItem.result.id);
                              },
                          },
                      ]
                    : []),
            ],
        });

        await menu.popup(new LogicalPosition(position.x, position.y));
    } catch (menuError) {
        actions.onError(`打开菜单失败：${String(menuError)}`);
    }
}
