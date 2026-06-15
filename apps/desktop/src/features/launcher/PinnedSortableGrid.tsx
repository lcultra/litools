import { move } from '@dnd-kit/helpers';
import { DragDropProvider } from '@dnd-kit/solid';
import { useSortable } from '@dnd-kit/solid/sortable';
import { createEffect, createSignal, For } from 'solid-js';
import { ResultIcon } from '../../components/ResultIcon';
import { preventDefault } from '../../shared/events';
import { HighlightedText } from './HighlightedText';
import type { LauncherRenderItem } from './LauncherView';

type PinnedSortableGridProps = {
    items: LauncherRenderItem[];
    selectedIndex: number;
    onDragEnd: () => void;
    onReorder: (orderedIds: string[]) => void;
    onResultClick: (item: LauncherRenderItem) => void;
    onResultContextMenu: (item: LauncherRenderItem, event: MouseEvent) => void;
    onSelectedElement: (element: HTMLElement) => void;
};

type SortablePinnedItem = LauncherRenderItem & { id: string };
type DragEndEvent = Parameters<typeof move>[1];

type SortablePinnedTileProps = {
    item: SortablePinnedItem;
    index: number;
    selected: boolean;
    suppressClickId: string | null;
    onResultClick: (item: LauncherRenderItem) => void;
    onResultContextMenu: (item: LauncherRenderItem, event: MouseEvent) => void;
    onSelectedElement: (element: HTMLElement) => void;
    onSuppressedClick: () => void;
};

function toSortableItems(items: LauncherRenderItem[]): SortablePinnedItem[] {
    return items.map((item) => ({ ...item, id: item.result.id }));
}

function hasSameItemSet(left: SortablePinnedItem[], right: SortablePinnedItem[]) {
    if (left.length !== right.length) {
        return false;
    }

    const ids = new Set(left.map((item) => item.id));
    return right.every((item) => ids.has(item.id));
}

function hasSameOrder(left: SortablePinnedItem[], right: SortablePinnedItem[]) {
    return left.length === right.length && left.every((item, index) => item.id === right[index]?.id);
}

function SortablePinnedTile(props: SortablePinnedTileProps) {
    let tileElement: HTMLElement | undefined;
    const sortable = useSortable({
        get id() {
            return props.item.id;
        },
        get index() {
            return props.index;
        },
    });

    createEffect(() => {
        if (props.selected && tileElement) {
            props.onSelectedElement(tileElement);
        }
    });

    function handleClick() {
        if (props.suppressClickId === props.item.id) {
            props.onSuppressedClick();
            return;
        }

        props.onResultClick(props.item);
    }

    return (
        // biome-ignore lint/a11y/useSemanticElements lint/a11y/useFocusableInteractive: dnd-kit sortable must bind directly to this div tile.
        <div
            ref={(element) => {
                tileElement = element;
                sortable.ref(element);
            }}
            class="relative grid size-full cursor-pointer grid-rows-[1fr_auto] place-items-center gap-1.5 rounded-2xl border border-transparent p-2 text-center text-inherit outline-none transition-[opacity,background-color,border-color] duration-150"
            classList={{
                'border-primary/40 bg-primary/15 text-text': props.selected && !sortable.isDragging(),
                'bg-transparent hover:bg-surface-hover/60 focus-visible:bg-surface-hover/60': !props.selected && !sortable.isDragging(),
                'pointer-events-none opacity-45 will-change-transform': sortable.isDragging(),
                'border-primary/70': sortable.isDropTarget() && !sortable.isDragging(),
            }}
            data-interactive
            onClick={handleClick}
            onContextMenu={(event) => props.onResultContextMenu(props.item, event)}
            onKeyDown={(event) => {
                if (event.key === 'Enter' || event.key === ' ') {
                    preventDefault(event);
                    handleClick();
                }
            }}
            role="button"
        >
            <ResultIcon result={props.item.result} />
            <HighlightedText class="w-full truncate text-xs font-medium" ranges={props.item.result.matches?.title} text={props.item.result.title} />
        </div>
    );
}

export function PinnedSortableGrid(props: PinnedSortableGridProps) {
    const [items, setItems] = createSignal<SortablePinnedItem[]>(toSortableItems(props.items));
    const [suppressClickId, setSuppressClickId] = createSignal<string | null>(null);

    createEffect(() => {
        const nextItems = toSortableItems(props.items);
        setItems((currentItems) => (hasSameItemSet(currentItems, nextItems) ? currentItems : nextItems));
    });

    function handleDragEnd(event: DragEndEvent) {
        const sourceId = event.operation.source?.id;

        if (sourceId !== undefined) {
            const id = String(sourceId);
            setSuppressClickId(id);
            window.setTimeout(() => {
                setSuppressClickId((currentId) => (currentId === id ? null : currentId));
            }, 200);
        }

        const currentItems = items();
        const nextItems = move(currentItems, event);

        if (!hasSameOrder(currentItems, nextItems)) {
            setItems(nextItems);
            props.onReorder(nextItems.map((item) => item.id));
        }

        props.onDragEnd();
    }

    return (
        <DragDropProvider onDragEnd={handleDragEnd}>
            <div class="grid auto-rows-[82px] grid-cols-9 gap-2">
                <For each={items()}>
                    {(item, index) => (
                        <SortablePinnedTile
                            item={item}
                            index={index()}
                            selected={props.selectedIndex === item.globalIndex}
                            suppressClickId={suppressClickId()}
                            onResultClick={props.onResultClick}
                            onResultContextMenu={props.onResultContextMenu}
                            onSelectedElement={props.onSelectedElement}
                            onSuppressedClick={() => setSuppressClickId(null)}
                        />
                    )}
                </For>
            </div>
        </DragDropProvider>
    );
}
