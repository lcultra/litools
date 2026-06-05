import { createSignal, For, onCleanup, Show } from 'solid-js';
import type { LauncherItem, SearchResult } from '../../bridge/types';
import { WindowFrame } from '../../components/WindowFrame';
import { providerLabel } from '../../shared/labels';
import { PaletteSearchInput } from './PaletteSearchInput';
import { PinnedSortableGrid } from './PinnedSortableGrid';

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
    onInputBlur: () => void;
    onKeyDown: (event: KeyboardEvent) => void;
    onPinnedDragEnd: () => void;
    onPinnedReorder: (orderedIds: string[]) => void;
    onResultContextMenu: (renderItem: PaletteRenderItem, position: { x: number; y: number }) => void;
    onResultRun: (result: SearchResult) => void;
    onSectionExpandedToggle: (sectionId: string) => void;
    onSubmit: (event: SubmitEvent) => void;
    query: string;
    renderSections: PaletteRenderSection[];
    selectedIndex: number;
};

type ResultButtonProps = {
    onClick: () => void;
    onContextMenu: (event: MouseEvent) => void;
    renderItem: PaletteRenderItem;
    selected: boolean;
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

function ResultButton(props: ResultButtonProps) {
    return (
        <button
            class="relative grid size-full cursor-pointer grid-rows-[1fr_auto] place-items-center gap-1.5 rounded-2xl border border-transparent p-2 text-center text-inherit outline-none transition-colors"
            classList={{
                'border-accent/40 bg-accent/15 text-fg': props.selected,
                'bg-transparent hover:bg-surface-muted/60 focus-visible:bg-surface-muted/60': !props.selected,
            }}
            draggable={false}
            onClick={props.onClick}
            onContextMenu={props.onContextMenu}
            tabindex={-1}
            type="button"
        >
            <ResultIcon result={props.renderItem.result} />
            <span class="w-full truncate text-xs font-medium">{props.renderItem.result.title}</span>
        </button>
    );
}

function stopInteractiveEvent(event: Event) {
    event.preventDefault();
    event.stopPropagation();
}

export function PaletteView(props: PaletteViewProps) {
    const totalVisibleItems = () => props.renderSections.reduce((count, section) => count + section.items.length, 0);
    const shouldShowResults = () => props.error || totalVisibleItems() > 0 || props.query.trim();

    function handlePanelContextMenu(event: MouseEvent) {
        event.preventDefault();
    }

    function handlePanelPointerDown(event: PointerEvent) {
        const target = event.target;

        if (!(target instanceof HTMLElement) || target.closest('input, textarea, [contenteditable="true"], [data-launcher-no-drag], [data-launcher-interactive]')) {
            return;
        }

        event.preventDefault();
    }

    function handleResultClick(renderItem: PaletteRenderItem) {
        props.onResultRun(renderItem.result);
    }

    function handleResultContextMenu(renderItem: PaletteRenderItem, event: MouseEvent) {
        event.preventDefault();
        event.stopPropagation();
        props.onResultContextMenu(renderItem, { x: event.clientX, y: event.clientY });
    }

    function handleContentElement(element: HTMLElement) {
        props.onContentElement(element);
        element.addEventListener('pointerdown', handlePanelPointerDown, { capture: true });
        onCleanup(() => element.removeEventListener('pointerdown', handlePanelPointerDown, { capture: true }));
    }

    return (
        <div on:contextmenu={handlePanelContextMenu}>
            <WindowFrame ref={handleContentElement} class="grid grid-rows-[auto_auto]">
                <PaletteSearchInput
                    inputRef={props.inputRef}
                    onInput={props.onInput}
                    onInputBlur={props.onInputBlur}
                    onKeyDown={props.onKeyDown}
                    onSubmit={props.onSubmit}
                    query={props.query}
                />

                <Show when={shouldShowResults()}>
                    <div class="max-h-[424px] min-h-0 overflow-y-auto overscroll-contain p-2">
                        <Show when={!props.error} fallback={<p class="m-0 px-4 py-3 text-sm text-danger">{props.error}</p>}>
                            <Show when={totalVisibleItems() > 0} fallback={<p class="m-0 px-4 py-3 text-sm text-muted">未找到结果</p>}>
                                <div class="grid gap-3">
                                    <For each={props.renderSections}>
                                        {(section) => (
                                            <section class="grid gap-2">
                                                <div class="flex h-6 items-center justify-between px-2 text-xs font-medium text-muted">
                                                    <span class="leading-none">{section.title}</span>
                                                    <div class="flex h-full items-center gap-2">
                                                        <Show when={section.totalCount > section.shownCount}>
                                                            <span>
                                                                {section.shownCount} / {section.totalCount}
                                                            </span>
                                                        </Show>
                                                        <Show when={section.canExpand && !section.expanded}>
                                                            <button
                                                                class="h-5 rounded-md px-2 text-xs leading-none text-accent hover:bg-surface-muted"
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

                                                <Show
                                                    when={section.id === 'pinned' && section.items.length > 1}
                                                    fallback={
                                                        <div class="grid auto-rows-[82px] grid-cols-9 gap-2">
                                                            <For each={section.items}>
                                                                {(renderItem) => (
                                                                    <ResultButton
                                                                        onClick={() => handleResultClick(renderItem)}
                                                                        onContextMenu={(event) => handleResultContextMenu(renderItem, event)}
                                                                        renderItem={renderItem}
                                                                        selected={props.selectedIndex === renderItem.globalIndex}
                                                                    />
                                                                )}
                                                            </For>
                                                        </div>
                                                    }
                                                >
                                                    <PinnedSortableGrid
                                                        items={section.items}
                                                        selectedIndex={props.selectedIndex}
                                                        onDragEnd={props.onPinnedDragEnd}
                                                        onReorder={props.onPinnedReorder}
                                                        onResultClick={handleResultClick}
                                                        onResultContextMenu={handleResultContextMenu}
                                                    />
                                                </Show>
                                            </section>
                                        )}
                                    </For>
                                </div>
                            </Show>
                        </Show>
                    </div>
                </Show>
            </WindowFrame>
        </div>
    );
}
