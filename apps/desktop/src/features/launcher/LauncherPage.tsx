import { useNavigate } from '@solidjs/router';
import { createEffect, createMemo, createSignal, onCleanup, onMount } from 'solid-js';
import { executeResult, hideMainWindow, launcherPanel, openPluginView, pinResult, reorderPinnedResults, resizeMainWindowHeight, unpinResult } from '../../bridge/commands';
import { onFocusSearch, onIndexStatusChanged } from '../../bridge/events';
import type { LauncherItem, LauncherSection, SearchResult } from '../../bridge/types';
import { type LauncherRenderSection, LauncherView } from './LauncherView';
import { useLauncherNavigation } from './useLauncherNavigation';
import { showResultContextMenu } from './useResultContextMenu';

const MIN_LAUNCHER_HEIGHT = 96;
const MAX_LAUNCHER_HEIGHT = 520;
export const PALETTE_GRID_COLUMNS = 9;
const DEFAULT_VISIBLE_ROWS = 2;

let cachedIdleSections: LauncherSection[] = [];

export type VisibleLauncherItem = {
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

export type VisibleRow = {
    items: VisibleLauncherItem[];
};

export function LauncherPage() {
    const navigate = useNavigate();
    let inputElement: HTMLInputElement | undefined;
    let resizeFrame = 0;
    let lastSyncedHeight = 0;
    let panelRequestId = 0;
    const [query, setQuery] = createSignal('');
    const [sections, setSections] = createSignal<LauncherSection[]>(cachedIdleSections);
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
            const summary = status.lastSummary as { success?: boolean } | undefined;
            if (summary?.success) {
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

            if (!currentQuery.trim()) {
                cachedIdleSections = response.sections;
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

    function parsePluginResultId(resultId: string): { pluginId: string; commandId: string } | null {
        // 格式: plugin:{plugin_id}:{command_id}
        if (!resultId.startsWith('plugin:')) return null;
        const parts = resultId.slice(7).split(':');
        if (parts.length < 2 || !parts[0] || !parts[1]) return null;
        return { pluginId: parts[0], commandId: parts[1] };
    }

    async function runResult(result: SearchResult | undefined) {
        const [firstAction] = result?.actions ?? [];

        if (!result || !firstAction) {
            return;
        }

        try {
            // 插件结果：直接调 openPluginView，不经过 executeResult
            const parsed = parsePluginResultId(result.id);
            if (parsed) {
                const info = await openPluginView(parsed.pluginId, parsed.commandId);
                setQuery('');
                setError(null);
                if (info.hostKind !== 'detached') {
                    const route = `/plugin/${parsed.pluginId}/${parsed.commandId}`;
                    navigate(route, { state: { runtimeId: info.runtimeId } });
                }
                return;
            }

            const response = await executeResult(result.id, firstAction.id, result.provider);
            setQuery('');
            setError(null);
            const effect = response.effect;
            if (typeof effect === 'object' && 'openPluginView' in effect) {
                // serde(rename_all) 在 enum 上只转换 variant 名，variant 内部字段保持 snake_case
                const ovp = (effect as Record<string, unknown>).openPluginView as Record<string, string> | undefined;
                const pluginId = ovp?.plugin_id;
                const commandId = ovp?.command_id;
                const route = ovp?.route;
                if (!pluginId || !commandId || !route) {
                    setError(`无效的插件参数：pluginId=${pluginId}, commandId=${commandId}, route=${route}`);
                    return;
                }
                const info = await openPluginView(pluginId, commandId);
                if (info.hostKind !== 'detached') {
                    navigate(route, { state: { runtimeId: info.runtimeId } });
                }
            } else {
                // 非插件结果：executeResult 已处理后端副作用，无需额外操作
            }
        } catch (executeError) {
            setError(`执行失败：${String(executeError)}`);
        }
    }

    async function reorderPinnedSection(resultIds: string[]) {
        setSections((currentSections) =>
            currentSections.map((section) => {
                if (section.id !== 'pinned') {
                    return section;
                }

                const itemsById = new Map(section.items.map((item) => [item.result.id, item]));
                const orderedItems = resultIds.flatMap((resultId) => {
                    const item = itemsById.get(resultId);
                    return item ? [item] : [];
                });

                if (orderedItems.length !== section.items.length) {
                    return section;
                }

                return { ...section, items: orderedItems };
            }),
        );

        try {
            await reorderPinnedResults(resultIds);
            setError(null);
        } catch (reorderError) {
            setError(`排序保存失败：${String(reorderError)}`);
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

    const navigation = useLauncherNavigation({
        inputElement: () => inputElement,
        visibleFlatItems,
        visibleRows,
        selectedIndex,
        setSelectedIndex,
        onEscape: () => {
            setQuery('');
            setError(null);
            void hideMainWindow();
        },
    });

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
        <LauncherView
            error={error()}
            inputRef={(element) => {
                inputElement = element;
            }}
            onContentElement={handleContentElement}
            onInput={setQuery}
            onKeyDown={navigation.handleKeyDown}
            onPinnedDragEnd={navigation.refocus}
            onPinnedReorder={(resultIds) => void reorderPinnedSection(resultIds)}
            onResultContextMenu={(renderItem, position) => {
                setSelectedIndex(renderItem.globalIndex);
                void showResultContextMenu(renderItem, position, {
                    onTogglePin: (item) => void togglePinned(item),
                    onError: (message) => setError(message),
                });
            }}
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
    const renderSections: LauncherRenderSection[] = [];
    const flatItems: VisibleLauncherItem[] = [];
    const rows: VisibleRow[] = [];

    sections.forEach((section, sectionIndex) => {
        const expanded = expandedSectionIds.has(section.id);
        const visibleCount = expanded ? section.items.length : Math.min(section.items.length, PALETTE_GRID_COLUMNS * DEFAULT_VISIBLE_ROWS);
        const rowOffset = rows.length;
        const visibleItems = section.items.slice(0, visibleCount).map((item, itemIndexInSection) => {
            const row = rowOffset + Math.floor(itemIndexInSection / PALETTE_GRID_COLUMNS);
            const col = itemIndexInSection % PALETTE_GRID_COLUMNS;
            const visibleItem: VisibleLauncherItem = {
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
