import { createSignal, For, Show } from 'solid-js';
import { startDragging } from '../../bridge/commands';
import type { LauncherItem, SearchResult } from '../../bridge/types';
import { Panel } from '../../components/Panel';
import { providerLabel } from '../../shared/labels';

export type PaletteRenderItem = {
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

export type PaletteRenderSection = {
    id: string;
    title: string;
    items: PaletteRenderItem[];
    shownCount: number;
    totalCount: number;
    expanded: boolean;
    canExpand: boolean;
};

type PaletteViewProps = {
    error: string | null;
    inputRef: (element: HTMLInputElement) => void;
    onContentElement: (element: HTMLElement) => void;
    onInput: (value: string) => void;
    onKeyDown: (event: KeyboardEvent) => void;
    onPinnedSectionReorder: (resultIds: string[]) => void;
    onResultContextMenu: (renderItem: PaletteRenderItem, position: { x: number; y: number }) => void;
    onResultRun: (result: SearchResult) => void;
    onSectionExpandedToggle: (sectionId: string) => void;
    onSubmit: (event: SubmitEvent) => void;
    query: string;
    renderSections: PaletteRenderSection[];
    selectedIndex: number;
};

function ResultIcon(props: { result: SearchResult }) {
    const [failed, setFailed] = createSignal(false);
    const fallback = () => providerLabel(props.result.provider).slice(0, 1);

    return (
        <span class="grid size-10 place-items-center overflow-hidden text-sm font-semibold text-muted">
            <Show when={props.result.iconUri && !failed()} fallback={fallback()}>
                <img alt="" class="size-10 object-contain" draggable={false} onError={() => setFailed(true)} src={props.result.iconUri ?? undefined} />
            </Show>
        </span>
    );
}

function stopInteractiveEvent(event: Event) {
    event.preventDefault();
    event.stopPropagation();
}

export function PaletteView(props: PaletteViewProps) {
    let draggedPinnedResultId: string | null = null;
    let suppressNextClickResultId: string | null = null;
    const totalVisibleItems = () => props.renderSections.reduce((count, section) => count + section.items.length, 0);
    const shouldShowResults = () => props.error || totalVisibleItems() > 0 || props.query.trim();

    function handleSearchDragStart(event: PointerEvent) {
        if (event.button !== 0) {
            return;
        }

        void startDragging();
    }

    function handlePanelContextMenu(event: MouseEvent) {
        event.preventDefault();
    }

    function pinnedResultIds() {
        return props.renderSections.find((section) => section.id === 'pinned')?.items.map((item) => item.result.id) ?? [];
    }

    function handlePinnedDragStart(renderItem: PaletteRenderItem, event: DragEvent) {
        if (renderItem.sectionId !== 'pinned') {
            return;
        }

        draggedPinnedResultId = renderItem.result.id;
        event.dataTransfer?.setData('text/plain', renderItem.result.id);
        event.dataTransfer?.setDragImage(event.currentTarget as Element, 24, 24);
    }

    function handlePinnedDragOver(renderItem: PaletteRenderItem, event: DragEvent) {
        if (renderItem.sectionId !== 'pinned' || !draggedPinnedResultId || draggedPinnedResultId === renderItem.result.id) {
            return;
        }

        event.preventDefault();
    }

    function handlePinnedDrop(renderItem: PaletteRenderItem, event: DragEvent) {
        if (renderItem.sectionId !== 'pinned' || !draggedPinnedResultId || draggedPinnedResultId === renderItem.result.id) {
            draggedPinnedResultId = null;
            return;
        }

        event.preventDefault();
        const nextIds = pinnedResultIds().filter((resultId) => resultId !== draggedPinnedResultId);
        const targetIndex = nextIds.indexOf(renderItem.result.id);

        if (targetIndex < 0) {
            draggedPinnedResultId = null;
            return;
        }

        nextIds.splice(targetIndex, 0, draggedPinnedResultId);
        suppressNextClickResultId = draggedPinnedResultId;
        draggedPinnedResultId = null;
        props.onPinnedSectionReorder(nextIds);
    }

    function handlePinnedDragEnd() {
        draggedPinnedResultId = null;
    }

    return (
        <div ref={props.onContentElement} class="p-px" on:contextmenu={handlePanelContextMenu}>
            <Panel class="grid w-full self-start grid-rows-[auto_auto]" variant="launcher">
                <form onPointerDown={handleSearchDragStart} onSubmit={props.onSubmit}>
                    <input
                        ref={props.inputRef}
                        autofocus
                        class="w-full border-0 border-b border-border bg-transparent px-6 py-6 text-2xl font-medium text-fg outline-none placeholder:text-muted"
                        id="launcher-search"
                        name="launcher-search"
                        on:keydown={props.onKeyDown}
                        onInput={(event) => props.onInput(event.currentTarget.value)}
                        placeholder="搜索应用、命令、文件、插件..."
                        value={props.query}
                    />
                </form>

                <Show when={shouldShowResults()}>
                    <div class="max-h-[424px] min-h-0 overflow-y-auto overscroll-contain p-2">
                        <Show when={!props.error} fallback={<p class="m-0 px-4 py-3 text-sm text-danger">{props.error}</p>}>
                            <Show when={totalVisibleItems() > 0} fallback={<p class="m-0 px-4 py-3 text-sm text-muted">未找到结果</p>}>
                                <div class="grid gap-3">
                                    <For each={props.renderSections}>
                                        {(section) => (
                                            <section class="grid gap-2">
                                                <div class="flex items-center justify-between px-2 text-xs font-medium text-muted">
                                                    <span>{section.title}</span>
                                                    <div class="flex items-center gap-2">
                                                        <Show when={section.totalCount > section.shownCount}>
                                                            <span>
                                                                {section.shownCount} / {section.totalCount}
                                                            </span>
                                                        </Show>
                                                        <Show when={section.canExpand && !section.expanded}>
                                                            <button
                                                                class="rounded-md px-2 py-1 text-xs text-accent hover:bg-surface-muted"
                                                                onClick={(event) => {
                                                                    stopInteractiveEvent(event);
                                                                    props.onSectionExpandedToggle(section.id);
                                                                }}
                                                                onPointerDown={stopInteractiveEvent}
                                                                data-no-drag
                                                                tabindex={-1}
                                                                type="button"
                                                            >
                                                                更多
                                                            </button>
                                                        </Show>
                                                    </div>
                                                </div>

                                                <div class="grid auto-rows-[82px] grid-cols-9 gap-2">
                                                    <For each={section.items}>
                                                        {(renderItem) => (
                                                            <button
                                                                class="grid size-full cursor-pointer grid-rows-[1fr_auto] place-items-center gap-1.5 rounded-2xl border border-transparent p-2 text-center text-inherit outline-none transition-colors"
                                                                classList={{
                                                                    'border-accent/40 bg-accent/15 text-fg': props.selectedIndex === renderItem.globalIndex,
                                                                    'bg-transparent hover:bg-surface-muted/60 focus-visible:bg-surface-muted/60':
                                                                        props.selectedIndex !== renderItem.globalIndex,
                                                                }}
                                                                draggable={renderItem.sectionId === 'pinned'}
                                                                onClick={() => {
                                                                    if (suppressNextClickResultId === renderItem.result.id) {
                                                                        suppressNextClickResultId = null;
                                                                        return;
                                                                    }

                                                                    props.onResultRun(renderItem.result);
                                                                }}
                                                                onContextMenu={(event) => {
                                                                    event.preventDefault();
                                                                    event.stopPropagation();
                                                                    props.onResultContextMenu(renderItem, { x: event.clientX, y: event.clientY });
                                                                }}
                                                                onDragEnd={handlePinnedDragEnd}
                                                                onDragOver={(event) => handlePinnedDragOver(renderItem, event)}
                                                                onDragStart={(event) => handlePinnedDragStart(renderItem, event)}
                                                                onDrop={(event) => handlePinnedDrop(renderItem, event)}
                                                                tabindex={-1}
                                                                type="button"
                                                            >
                                                                <ResultIcon result={renderItem.result} />
                                                                <span class="w-full truncate text-xs font-medium">{renderItem.result.title}</span>
                                                            </button>
                                                        )}
                                                    </For>
                                                </div>
                                            </section>
                                        )}
                                    </For>
                                </div>
                            </Show>
                        </Show>
                    </div>
                </Show>
            </Panel>
        </div>
    );
}
