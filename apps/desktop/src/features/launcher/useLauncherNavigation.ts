import type { Accessor, Setter } from 'solid-js';
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
        queueMicrotask(() => inputElement()?.focus({ preventScroll: true }));
    }

    function refocusAfterBlur() {
        focusSearchInput();
        window.requestAnimationFrame(() => inputElement()?.focus({ preventScroll: true }));
    }

    function refocusAfterDrag() {
        window.setTimeout(() => {
            void import('../../bridge/commands').then(({ focusMainWindow }) => focusMainWindow().then(() => inputElement()?.focus({ preventScroll: true })));
        }, 0);
    }

    function selectAdjacent(offset: number) {
        const items = visibleFlatItems();
        setSelectedIndex((current) => (items.length ? (current + offset + items.length) % items.length : 0));
        focusSearchInput();
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
        focusSearchInput();
    }

    function handleKeyDown(event: KeyboardEvent) {
        switch (event.key) {
            case 'Tab':
                event.preventDefault();
                event.stopPropagation();
                selectAdjacent(event.shiftKey ? -1 : 1);
                return;
            case 'ArrowRight':
                event.preventDefault();
                selectAdjacent(1);
                return;
            case 'ArrowLeft':
                event.preventDefault();
                selectAdjacent(-1);
                return;
            case 'ArrowDown':
                event.preventDefault();
                selectVertical(1);
                return;
            case 'ArrowUp':
                event.preventDefault();
                selectVertical(-1);
                return;
            case 'Escape':
                event.preventDefault();
                onEscape();
                return;
        }
    }

    return { focusSearchInput, handleKeyDown, refocusAfterBlur, refocusAfterDrag };
}
