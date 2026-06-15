import { createEffect, For, Show } from 'solid-js';
import type { LauncherItem, SearchResult } from '../../bridge/types';
import { ResultIcon } from '../../components/ResultIcon';
import { WindowFrame } from '../../components/WindowFrame';
import { stopEvent } from '../../shared/events';
import { HighlightedText } from './HighlightedText';
import { LauncherInput } from './LauncherInput';
import { PinnedSortableGrid } from './PinnedSortableGrid';

export type LauncherRenderItem = {
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

export type LauncherRenderSection = {
    id: string;
    title: string;
    items: LauncherRenderItem[];
    shownCount: number;
    totalCount: number;
    expanded: boolean;
    canExpand: boolean;
};

type LauncherViewProps = {
    error: string | null;
    inputRef: (element: HTMLInputElement) => void;
    onContentElement: (element: HTMLElement) => void;
    onInput: (value: string) => void;
    onKeyDown: (event: KeyboardEvent) => void;
    onPinnedDragEnd: () => void;
    onPinnedReorder: (orderedIds: string[]) => void;
    onResultContextMenu: (renderItem: LauncherRenderItem, position: { x: number; y: number }) => void;
    onResultRun: (result: SearchResult) => void;
    onSectionExpandedToggle: (sectionId: string) => void;
    onSubmit: (event: SubmitEvent) => void;
    query: string;
    renderSections: LauncherRenderSection[];
    selectedIndex: number;
};

type ResultButtonProps = {
    onClick: () => void;
    onContextMenu: (event: MouseEvent) => void;
    onSelectedElement?: (element: HTMLElement) => void;
    renderItem: LauncherRenderItem;
    selected: boolean;
};

function ResultButton(props: ResultButtonProps) {
    let buttonElement: HTMLButtonElement | undefined;

    createEffect(() => {
        if (props.selected && buttonElement) {
            props.onSelectedElement?.(buttonElement);
        }
    });

    return (
        <button
            ref={buttonElement}
            class="relative grid size-full cursor-pointer grid-rows-[1fr_auto] place-items-center gap-1.5 rounded-2xl border border-transparent p-2 text-center text-inherit outline-none transition-colors"
            classList={{
                'border-primary/40 bg-primary/15 text-text': props.selected,
                'bg-transparent hover:bg-surface-hover/60 focus-visible:bg-surface-hover/60': !props.selected,
            }}
            draggable={false}
            onClick={props.onClick}
            onContextMenu={props.onContextMenu}
            tabindex={-1}
            type="button"
        >
            <ResultIcon result={props.renderItem.result} />
            <HighlightedText class="w-full truncate text-xs font-medium" ranges={props.renderItem.result.matches?.title} text={props.renderItem.result.title} />
        </button>
    );
}

export function LauncherView(props: LauncherViewProps) {
    let selectedResultElement: HTMLElement | undefined;
    const totalVisibleItems = () => props.renderSections.reduce((count, section) => count + section.items.length, 0);
    const shouldShowResults = () => props.error || totalVisibleItems() > 0 || props.query.trim();

    createEffect(() => {
        props.selectedIndex;
        props.renderSections;

        queueMicrotask(() => {
            selectedResultElement?.scrollIntoView({ block: 'nearest' });
        });
    });

    // 阻止 mousedown 默认行为以保持输入框焦点不被转移。
    // 输入框自身除外——保留选文本、移动光标等操作。
    function handleMouseDown(event: MouseEvent) {
        const target = event.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;
        event.preventDefault();
    }

    function handleFocusIn(event: FocusEvent) {
        const target = event.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;
        event.preventDefault();
    }

    function handleResultClick(renderItem: LauncherRenderItem) {
        props.onResultRun(renderItem.result);
    }

    function handleResultContextMenu(renderItem: LauncherRenderItem, event: MouseEvent) {
        stopEvent(event);
        props.onResultContextMenu(renderItem, { x: event.clientX, y: event.clientY });
    }

    function handleContentElement(element: HTMLElement) {
        props.onContentElement(element);
    }

    return (
        <WindowFrame ref={handleContentElement} class="grid grid-rows-[auto_auto]">
            <LauncherInput inputRef={props.inputRef} onInput={props.onInput} onKeyDown={props.onKeyDown} onSubmit={props.onSubmit} query={props.query} />

            <Show when={shouldShowResults()}>
                <div class="max-h-[424px] min-h-0 overflow-y-auto overscroll-contain p-2">
                    <Show when={!props.error} fallback={<p class="m-0 px-4 py-3 text-sm text-danger">{props.error}</p>}>
                        <Show when={totalVisibleItems() > 0} fallback={<p class="m-0 px-4 py-3 text-sm text-text-muted">未找到结果</p>}>
                            {/* biome-ignore lint/a11y/noStaticElementInteractions: mousedown/focusin handler 仅用于阻止焦点转移，非用户交互功能 */}
                            <div class="grid gap-3" role="presentation" onMouseDown={handleMouseDown} onFocusIn={handleFocusIn}>
                                <For each={props.renderSections}>
                                    {(section) => (
                                        <section class="grid gap-2">
                                            <div class="flex h-6 items-center justify-between px-2 text-xs font-medium text-text-muted">
                                                <span class="leading-none">{section.title}</span>
                                                <div class="flex h-full items-center gap-2">
                                                    <Show when={section.totalCount > section.shownCount}>
                                                        <span>
                                                            {section.shownCount} / {section.totalCount}
                                                        </span>
                                                    </Show>
                                                    <Show when={section.canExpand && !section.expanded}>
                                                        <button
                                                            class="h-5 rounded-md px-2 text-xs leading-none text-primary hover:bg-surface-hover"
                                                            onClick={(event) => {
                                                                stopEvent(event);
                                                                props.onSectionExpandedToggle(section.id);
                                                            }}
                                                            onPointerDown={stopEvent}
                                                            data-interactive
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
                                                                    onSelectedElement={(element) => {
                                                                        if (props.selectedIndex === renderItem.globalIndex) {
                                                                            selectedResultElement = element;
                                                                        }
                                                                    }}
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
                                                    onSelectedElement={(element) => {
                                                        selectedResultElement = element;
                                                    }}
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
    );
}
