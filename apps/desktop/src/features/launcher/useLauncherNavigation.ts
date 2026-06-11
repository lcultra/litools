import type { Accessor, Setter } from 'solid-js';
import { preventDefault } from '../../shared/events';
import type { VisibleLauncherItem, VisibleRow } from './LauncherPage';

type LauncherNavigationOptions = {
    inputElement: Accessor<HTMLInputElement | undefined>;
    visibleFlatItems: Accessor<VisibleLauncherItem[]>;
    visibleRows: Accessor<VisibleRow[]>;
    selectedIndex: Accessor<number>;
    setSelectedIndex: Setter<number>;
    onEscape: () => void;
};

export function useLauncherNavigation(options: LauncherNavigationOptions) {
    const { inputElement, visibleFlatItems, visibleRows, selectedIndex, setSelectedIndex, onEscape } = options;

    function focusSearchInput() {
        inputElement()?.focus({ preventScroll: true });
    }

    // 拖拽结束后恢复焦点（dnd-kit 排序 / 窗口拖拽）
    function refocus() {
        queueMicrotask(() => inputElement()?.focus({ preventScroll: true }));
    }

    function selectAdjacent(offset: number) {
        const items = visibleFlatItems();
        setSelectedIndex((current) => (items.length ? (current + offset + items.length) % items.length : 0));
    }

    function selectVertical(offset: number) {
        const items = visibleFlatItems();
        const rows = visibleRows();

        if (!items.length || !rows.length) {
            setSelectedIndex(0);
            return;
        }

        const current = items[selectedIndex()] ?? items[0];
        const currentRowIndex = rows.findIndex((row: VisibleRow) => row.items.some((item: VisibleLauncherItem) => item.globalIndex === current.globalIndex));
        const nextRowIndex = (currentRowIndex + offset + rows.length) % rows.length;
        const targetRow = rows[nextRowIndex];
        const targetItem = targetRow.items[Math.min(current.col, targetRow.items.length - 1)];

        setSelectedIndex(targetItem.globalIndex);
    }

    function handleKeyDown(event: KeyboardEvent) {
        switch (event.key) {
            case 'Tab':
                preventDefault(event);
                selectAdjacent(event.shiftKey ? -1 : 1);
                return;
            case 'ArrowRight':
                preventDefault(event);
                selectAdjacent(1);
                return;
            case 'ArrowLeft':
                preventDefault(event);
                selectAdjacent(-1);
                return;
            case 'ArrowDown':
                preventDefault(event);
                selectVertical(1);
                return;
            case 'ArrowUp':
                preventDefault(event);
                selectVertical(-1);
                return;
            case 'Escape':
                preventDefault(event);
                onEscape();
                return;
        }
    }

    return { focusSearchInput, handleKeyDown, refocus };
}
