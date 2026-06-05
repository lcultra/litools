import { move } from '@dnd-kit/helpers';
import { DragDropProvider } from '@dnd-kit/solid';
import { useSortable } from '@dnd-kit/solid/sortable';
import { createEffect, createSignal, For, Show } from 'solid-js';
import type { SearchResult } from '../../bridge/types';
import { providerLabel } from '../../shared/labels';
import type { PaletteRenderItem } from './PaletteView';

type PinnedSortableGridProps = {
    items: PaletteRenderItem[];
    selectedIndex: number;
    onReorder: (orderedIds: string[]) => void;
    onResultClick: (item: PaletteRenderItem) => void;
    onResultContextMenu: (item: PaletteRenderItem, event: MouseEvent) => void;
};

type SortablePinnedItem = PaletteRenderItem & { id: string };
type DragEndEvent = Parameters<typeof move>[1];

type SortablePinnedTileProps = {
    item: SortablePinnedItem;
    index: number;
    selected: boolean;
    suppressClickId: string | null;
    onResultClick: (item: PaletteRenderItem) => void;
    onResultContextMenu: (item: PaletteRenderItem, event: MouseEvent) => void;
    onSuppressedClick: () => void;
};

function SortableItemIcon(props: { result: SearchResult }) {
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

function toSortableItems(items: PaletteRenderItem[]): SortablePinnedItem[] {
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
    const sortable = useSortable({
        get id() {
            return props.item.id;
        },
        get index() {
            return props.index;
        },
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
            ref={sortable.ref}
            class="relative grid size-full cursor-grab grid-rows-[1fr_auto] place-items-center gap-1.5 rounded-2xl border border-transparent p-2 text-center text-inherit outline-none transition-[opacity,background-color,border-color] duration-150 active:cursor-grabbing"
            classList={{
                'border-accent/40 bg-accent/15 text-fg': props.selected && !sortable.isDragging(),
                'bg-transparent hover:bg-surface-muted/60 focus-visible:bg-surface-muted/60': !props.selected && !sortable.isDragging(),
                'pointer-events-none opacity-45 will-change-transform': sortable.isDragging(),
                'border-accent/70': sortable.isDropTarget() && !sortable.isDragging(),
            }}
            onClick={handleClick}
            onContextMenu={(event) => props.onResultContextMenu(props.item, event)}
            onKeyDown={(event) => {
                if (event.key === 'Enter' || event.key === ' ') {
                    event.preventDefault();
                    handleClick();
                }
            }}
            role="button"
            tabindex={-1}
        >
            <SortableItemIcon result={props.item.result} />
            <span class="w-full truncate text-xs font-medium">{props.item.result.title}</span>
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

        if (hasSameOrder(currentItems, nextItems)) {
            return;
        }

        setItems(nextItems);
        props.onReorder(nextItems.map((item) => item.id));
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
                            onSuppressedClick={() => setSuppressClickId(null)}
                        />
                    )}
                </For>
            </div>
        </DragDropProvider>
    );
}
