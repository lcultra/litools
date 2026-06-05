import { LogicalPosition } from '@tauri-apps/api/dpi';
import { Menu } from '@tauri-apps/api/menu';
import { createEffect, createMemo, createSignal, onCleanup, onMount } from 'solid-js';
import { executeResult, hideMainWindow, launcherPanel, pinResult, reorderPinnedResults, resizeMainWindowHeight, unpinResult } from '../../bridge/commands';
import { onFocusSearch, onIndexStatusChanged } from '../../bridge/events';
import type { BuiltinCommandEffect, LauncherItem, LauncherSection, SearchResult } from '../../bridge/types';
import { type PaletteRenderSection, PaletteView } from './PaletteView';

const MIN_LAUNCHER_HEIGHT = 96;
const MAX_LAUNCHER_HEIGHT = 520;
export const PALETTE_GRID_COLUMNS = 9;
const DEFAULT_VISIBLE_ROWS = 2;

type PaletteSearchControllerProps = {
    onCommandEffect: (effect: BuiltinCommandEffect) => void;
};

export type VisiblePaletteItem = {
    item: LauncherItem;
    result: SearchResult;
    sectionId: string;
    globalIndex: number;
    sectionIndex: number;
    itemIndexInSection: number;
    row: number;
    col: number;
    isPinned: boolean;
};

type VisibleRow = {
    items: VisiblePaletteItem[];
};

export function PaletteSearchController(props: PaletteSearchControllerProps) {
    let inputElement: HTMLInputElement | undefined;
    let resizeFrame = 0;
    let lastSyncedHeight = 0;
    let panelRequestId = 0;
    const [query, setQuery] = createSignal('');
    const [sections, setSections] = createSignal<LauncherSection[]>([]);
    const [expandedSectionIds, setExpandedSectionIds] = createSignal<Set<string>>(new Set<string>());
    const [selectedIndex, setSelectedIndex] = createSignal(0);
    const [error, setError] = createSignal<string | null>(null);

    const renderModel = createMemo(() => buildRenderModel(sections(), expandedSectionIds()));
    const visibleFlatItems = createMemo(() => renderModel().flatItems);
    const visibleRows = createMemo(() => renderModel().rows);

    onMount(() => {
        inputElement?.focus();
        const unsubscribe = onFocusSearch(() => {
            inputElement?.focus();
            inputElement?.select();
        });
        const unsubscribeIndexStatus = onIndexStatusChanged((status) => {
            if (status.lastSummary?.success) {
                void refreshPanel({ preserveSelection: true, resetExpansion: false });
            }
        });

        onCleanup(() => {
            void unsubscribe.then((dispose) => dispose());
            void unsubscribeIndexStatus.then((dispose) => dispose());
        });
    });

    createEffect(() => {
        query();
        const timeout = window.setTimeout(() => {
            void refreshPanel({ preserveSelection: false, resetExpansion: true });
        }, 120);

        onCleanup(() => window.clearTimeout(timeout));
    });

    createEffect(() => {
        const itemCount = visibleFlatItems().length;
        setSelectedIndex((current) => (itemCount ? Math.min(current, itemCount - 1) : 0));
    });

    async function refreshPanel(options: { preserveSelection: boolean; resetExpansion: boolean }) {
        const currentQuery = query();
        const previousSelectedId = visibleFlatItems()[selectedIndex()]?.result.id;
        const requestId = ++panelRequestId;
        setError(null);

        try {
            const response = await launcherPanel(currentQuery);
            if (requestId !== panelRequestId) {
                return;
            }

            setSections(response.sections);
            if (options.resetExpansion) {
                setExpandedSectionIds(new Set<string>());
            }

            if (options.preserveSelection && previousSelectedId) {
                queueMicrotask(() => {
                    const nextSelectedIndex = visibleFlatItems().findIndex((item) => item.result.id === previousSelectedId);
                    setSelectedIndex(nextSelectedIndex >= 0 ? nextSelectedIndex : 0);
                });
            } else {
                setSelectedIndex(0);
            }
        } catch (searchError) {
            if (requestId === panelRequestId) {
                setError(`搜索失败：${String(searchError)}`);
            }
        }
    }

    async function runResult(result: SearchResult | undefined) {
        const [firstAction] = result?.actions ?? [];

        if (!result || !firstAction) {
            return;
        }

        try {
            const response = await executeResult(result.id, firstAction.id);
            setQuery('');
            setError(null);
            props.onCommandEffect(response.effect);
        } catch (executeError) {
            setError(`执行失败：${String(executeError)}`);
        }
    }

    async function togglePinned(item: LauncherItem) {
        const pinned = item.isPinned;

        try {
            if (pinned) {
                await unpinResult(item.result.id);
            } else {
                await pinResult(item.result.id);
            }

            await refreshPanel({ preserveSelection: true, resetExpansion: false });
        } catch (pinError) {
            setError(`${pinned ? '取消固定' : '固定'}失败：${String(pinError)}`);
        }
    }

    async function showResultContextMenu(renderItem: VisiblePaletteItem, position: { x: number; y: number }) {
        setSelectedIndex(renderItem.globalIndex);

        try {
            const menu = await Menu.new({
                items: [
                    {
                        id: renderItem.item.isPinned ? 'unpin-from-search-bar' : 'pin-to-search-bar',
                        text: renderItem.item.isPinned ? '从搜索栏取消固定' : '固定到搜索栏',
                        action: () => void togglePinned(renderItem.item),
                    },
                ],
            });

            await menu.popup(new LogicalPosition(position.x, position.y));
        } catch (menuError) {
            setError(`打开菜单失败：${String(menuError)}`);
        }
    }

    async function reorderPinnedSection(resultIds: string[]) {
        try {
            await reorderPinnedResults(resultIds);
            await refreshPanel({ preserveSelection: true, resetExpansion: false });
        } catch (reorderError) {
            setError(`排序失败：${String(reorderError)}`);
        }
    }

    function handleSubmit(event: SubmitEvent) {
        event.preventDefault();
        void runResult(visibleFlatItems()[selectedIndex()]?.result);
    }

    function clampLauncherHeight(height: number) {
        return Math.min(Math.max(Math.ceil(height), MIN_LAUNCHER_HEIGHT), MAX_LAUNCHER_HEIGHT);
    }

    function scheduleWindowHeightSync(height: number) {
        const nextHeight = clampLauncherHeight(height);

        if (nextHeight === lastSyncedHeight) {
            return;
        }

        window.cancelAnimationFrame(resizeFrame);
        resizeFrame = window.requestAnimationFrame(() => {
            lastSyncedHeight = nextHeight;
            void resizeMainWindowHeight(nextHeight);
        });
    }

    function measureContentHeight(element: HTMLElement) {
        return element.getBoundingClientRect().height;
    }

    function handleContentElement(element: HTMLElement) {
        const observer = new ResizeObserver(() => scheduleWindowHeightSync(measureContentHeight(element)));
        observer.observe(element);
        scheduleWindowHeightSync(measureContentHeight(element));

        onCleanup(() => {
            window.cancelAnimationFrame(resizeFrame);
            observer.disconnect();
        });
    }

    function selectAdjacent(offset: number) {
        const items = visibleFlatItems();
        setSelectedIndex((current) => (items.length ? (current + offset + items.length) % items.length : 0));
        queueMicrotask(() => inputElement?.focus());
    }

    function selectVertical(offset: number) {
        const items = visibleFlatItems();
        const rows = visibleRows();

        if (!items.length || !rows.length) {
            setSelectedIndex(0);
            return;
        }

        const current = items[selectedIndex()] ?? items[0];
        const currentRowIndex = rows.findIndex((row) => row.items.some((item) => item.globalIndex === current.globalIndex));
        const nextRowIndex = (currentRowIndex + offset + rows.length) % rows.length;
        const targetRow = rows[nextRowIndex];
        const targetItem = targetRow.items[Math.min(current.col, targetRow.items.length - 1)];

        setSelectedIndex(targetItem.globalIndex);
        queueMicrotask(() => inputElement?.focus());
    }

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key === 'Tab') {
            event.preventDefault();
            event.stopPropagation();
            selectAdjacent(event.shiftKey ? -1 : 1);
            return;
        }

        if (event.key === 'ArrowRight') {
            event.preventDefault();
            selectAdjacent(1);
            return;
        }

        if (event.key === 'ArrowLeft') {
            event.preventDefault();
            selectAdjacent(-1);
            return;
        }

        if (event.key === 'ArrowDown') {
            event.preventDefault();
            selectVertical(1);
            return;
        }

        if (event.key === 'ArrowUp') {
            event.preventDefault();
            selectVertical(-1);
            return;
        }

        if (event.key === 'Escape') {
            event.preventDefault();
            setQuery('');
            setError(null);
            void hideMainWindow();
        }
    }

    function toggleSectionExpanded(sectionId: string) {
        setExpandedSectionIds((current) => {
            const next = new Set(current);

            if (next.has(sectionId)) {
                next.delete(sectionId);
            } else {
                next.add(sectionId);
            }

            return next;
        });
    }

    return (
        <PaletteView
            error={error()}
            inputRef={(element) => {
                inputElement = element;
            }}
            onContentElement={handleContentElement}
            onInput={setQuery}
            onKeyDown={handleKeyDown}
            onPinnedSectionReorder={(resultIds) => void reorderPinnedSection(resultIds)}
            onResultContextMenu={(renderItem, position) => void showResultContextMenu(renderItem, position)}
            onResultRun={(result) => void runResult(result)}
            onSectionExpandedToggle={toggleSectionExpanded}
            onSubmit={handleSubmit}
            query={query()}
            renderSections={renderModel().sections}
            selectedIndex={selectedIndex()}
        />
    );
}

function buildRenderModel(sections: LauncherSection[], expandedSectionIds: Set<string>) {
    const renderSections: PaletteRenderSection[] = [];
    const flatItems: VisiblePaletteItem[] = [];
    const rows: VisibleRow[] = [];

    sections.forEach((section, sectionIndex) => {
        const expanded = expandedSectionIds.has(section.id);
        const visibleCount = expanded ? section.items.length : Math.min(section.items.length, PALETTE_GRID_COLUMNS * DEFAULT_VISIBLE_ROWS);
        const rowOffset = rows.length;
        const visibleItems = section.items.slice(0, visibleCount).map((item, itemIndexInSection) => {
            const row = rowOffset + Math.floor(itemIndexInSection / PALETTE_GRID_COLUMNS);
            const col = itemIndexInSection % PALETTE_GRID_COLUMNS;
            const visibleItem: VisiblePaletteItem = {
                item,
                result: item.result,
                sectionId: section.id,
                globalIndex: flatItems.length,
                sectionIndex,
                itemIndexInSection,
                row,
                col,
                isPinned: item.isPinned,
            };
            flatItems.push(visibleItem);

            if (!rows[row]) {
                rows[row] = { items: [] };
            }
            rows[row].items.push(visibleItem);

            return visibleItem;
        });

        renderSections.push({
            id: section.id,
            title: section.title,
            items: visibleItems,
            shownCount: visibleItems.length,
            totalCount: section.items.length,
            expanded,
            canExpand: section.items.length > PALETTE_GRID_COLUMNS * DEFAULT_VISIBLE_ROWS,
        });
    });

    return { sections: renderSections, flatItems, rows };
}
